// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    not(any(debug_assertions, feature = "devtools")),
    windows_subsystem = "windows"
)]
#![feature(lazy_cell)]

use std::error::Error;
use std::path::PathBuf;
use std::{env, vec};

use anyhow::{bail, Context, Result};
use app::AppState;
use epub::rootfile::EpubRootfile;
use epub::toc::EpubToc;
use error::CommandError;
use tauri::api::dialog;
use tauri::http::{Request, Response};
use tauri::{App, AppHandle, Manager, WindowBuilder, WindowUrl, Wry};
use typed_path::Utf8NativePathBuf;
use utils::now_unix_timestamp;

pub mod app;
pub mod epub;
pub mod error;
pub mod library;
pub mod renderer;
pub mod utils;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            if let Err(e) = app_setup(app) {
                let message = format!("Failed to initialize app:\n{:?}", e);
                dialog::message::<Wry>(None, "Error", message);
            }
            Ok(())
        })
        .register_uri_scheme_protocol("book", book_protocol)
        .invoke_handler(tauri::generate_handler![
            get_toc,
            get_rootfile,
            get_progress,
            save_progress,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn app_setup(app: &mut App) -> Result<()> {
    let port = renderer::start_http_server(app.handle())?;
    let state = AppState::init(port)?;
    app.manage(state);

    let handle = app.handle();
    std::thread::spawn(move || {
        let args = env::args_os().skip(1);
        if args.len() == 0 {
            // TODO: open library
            println!("No book specified");
            return;
        }

        for arg in args {
            let path = PathBuf::from(arg);
            if let Err(e) = open_book_window(handle.clone(), path) {
                let message = format!("{:?}", e);
                dialog::message::<Wry>(None, "Error", message);
            }
        }
    });

    Ok(())
}

fn book_protocol(_app: &AppHandle, _req: &Request) -> Result<Response, Box<dyn Error>> {
    unimplemented!()
}

fn open_book_window(app: AppHandle, path: PathBuf) -> Result<()> {
    let path = match path.into_os_string().into_string() {
        Ok(path) => Utf8NativePathBuf::from(path),
        Err(path) => bail!("Invalid path:\n{}", path.to_string_lossy()),
    };

    let state = app.state::<AppState>();
    let id = state.open_book(path)?;
    let port = state.renderer_port();

    match app.get_window(&id) {
        Some(window) => window.set_focus().context("Failed to focus reader window"),
        None => {
            let url = WindowUrl::App("index.html".into());

            /*
            // TODO: use custom protocol after tauri-apps/tauri#6536 resolved
            #[cfg(windows)]
            let origin = "https://book.localhost";
            #[cfg(unix)]
            let origin = "book://localhost";
            */

            let script = format!(
                "
                const BOOK_ID = '{id}';
                const RENDERER = 'http://127.0.0.1:{port}';
                "
            );

            let window = WindowBuilder::new(&app, id, url)
                .title("Ellisia")
                .min_inner_size(900.0, 800.0)
                .inner_size(1200.0, 900.0)
                .position(60.0, 60.0)
                .focused(true)
                .initialization_script(&script)
                .build()
                .context("Failed to create reader window")?;

            #[cfg(any(debug_assertions, feature = "devtools"))]
            window.open_devtools();

            Ok(())
        }
    }
}

#[tauri::command]
fn get_toc(app: AppHandle, id: &str) -> Result<EpubToc, CommandError> {
    let state = app.state::<AppState>();
    let epub = state.epubs().get(id).context("Book not opened")?;
    let toc = epub.toc().clone();
    Ok(toc)
}

#[tauri::command]
fn get_rootfile(app: AppHandle, id: &str) -> Result<EpubRootfile, CommandError> {
    let state = app.state::<AppState>();
    let epub = state.epubs().get(id).context("Book not opened")?;
    let toc = epub.rootfile().clone();
    Ok(toc)
}

#[tauri::command]
fn get_progress(app: AppHandle, id: &str) -> Result<(String, f64), CommandError> {
    let state = app.state::<AppState>();
    let mut library = state.library().lock();

    let (path, progress) = match library.books_mut().get(id) {
        Some(book) => (book.content_path.clone(), book.content_progress),
        None => ("".to_string(), 0.0),
    };

    Ok((path, progress))
}

#[tauri::command]
fn save_progress(app: AppHandle, id: &str, path: &str, progress: f64) -> Result<(), CommandError> {
    let state = app.state::<AppState>();
    let mut library = state.library().lock();

    match library.books_mut().get_mut(id) {
        Some(mut book) => {
            book.content_path = path.to_string();
            book.content_progress = progress;
            book.last_read_at = now_unix_timestamp();
        }
        None => {
            // Should not happen
        }
    }

    library.persist()?;

    Ok(())
}
