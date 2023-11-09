// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    not(any(debug_assertions, feature = "devtools")),
    windows_subsystem = "windows"
)]
#![feature(lazy_cell)]

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::json;
use state::AppState;
use tauri::api::dialog;
use tauri::http::{Request, Response};
use tauri::{App, AppHandle, Manager, WindowBuilder, WindowEvent, WindowUrl, Wry};
use typed_path::Utf8NativePathBuf;
use utils::{get_config_dir_path, init_dir};

pub mod commands;
pub mod epub;
pub mod error;
pub mod library;
pub mod renderer;
pub mod state;
pub mod utils;
pub mod zip;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            if let Err(e) = app_setup(app) {
                let message = format!("Failed to initialize app:\n{:?}", e);
                dialog::message::<Wry>(None, "Error", message);
            }
            Ok(())
        })
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);
            let argv = argv.into_iter().map(OsString::from).collect();
            let cwd = PathBuf::from(cwd);
            launch(app.clone(), argv, cwd);
        }))
        .register_uri_scheme_protocol("book", book_protocol)
        .invoke_handler(tauri::generate_handler![
            commands::open_book,
            commands::open_library,
            commands::close_library,
            commands::get_library,
            commands::get_toc,
            commands::get_rootfile,
            commands::get_progress,
            commands::save_progress,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn app_setup(app: &mut App) -> Result<()> {
    let dir = get_config_dir_path()?;
    init_dir(&dir)?;
    init_dir(&dir.join("cover"))?;

    let port = renderer::start_http_server(app.handle())?;
    let state = AppState::init(port)?;
    app.manage(state);

    let handle = app.handle();
    let args = env::args_os().collect();
    let cwd = std::env::current_dir().unwrap_or_default();
    launch(handle, args, cwd);

    Ok(())
}

fn book_protocol(_app: &AppHandle, _req: &Request) -> Result<Response, Box<dyn Error>> {
    unimplemented!()
}

fn launch(app: AppHandle, args: Vec<OsString>, cwd: PathBuf) {
    let args = args.into_iter().skip(1);
    if args.len() == 0 {
        if let Err(e) = launch_library(app) {
            let message = format!("{:?}", e);
            dialog::message::<Wry>(None, "Error", message);
        }
        return;
    }

    for arg in args {
        let path = match cwd.join(arg).canonicalize() {
            Ok(path) => path,
            Err(e) => {
                let message = format!("Failed to resolve path:\n{}", e);
                dialog::message::<Wry>(None, "Error", message);
                continue;
            }
        };

        let path = match path.into_os_string().into_string() {
            Ok(path) => Utf8NativePathBuf::from(path),
            Err(path) => {
                let message = format!("Invalid path:\n{}", path.to_string_lossy());
                dialog::message::<Wry>(None, "Error", message);
                continue;
            }
        };

        if let Err(e) = launch_book(app.clone(), path) {
            let message = format!("{:?}", e);
            dialog::message::<Wry>(None, "Error", message);
        }
    }
}

fn launch_library(app: AppHandle) -> Result<()> {
    let state = app.state::<AppState>();
    let port = state.renderer_port();

    match app.get_window("library") {
        Some(window) => window.set_focus().context("Failed to focus library window"),
        None => {
            let url = WindowUrl::App("index.html".into());

            let config = json!({
                "renderer": format!("http://127.0.0.1:{port}"),
            });

            let window = WindowBuilder::new(&app, "library", url)
                .title("Library - Ellisia")
                .min_inner_size(900.0, 800.0)
                .inner_size(1200.0, 900.0)
                .center()
                .focused(true)
                .initialization_script(&format!("const ELLISIA = {config};"))
                .build()
                .context("Failed to create library window")?;

            #[cfg(any(debug_assertions, feature = "devtools"))]
            window.open_devtools();

            Ok(())
        }
    }
}

/// `path` must be canonicalized before calling this function.
fn launch_book(app: AppHandle, path: Utf8NativePathBuf) -> Result<()> {
    let state = app.state::<AppState>();
    let port = state.renderer_port();
    let (id, book) = state.open_book(path.clone())?;

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

            let title = match book.metadata.title {
                Some(title) => format!("{} - Ellisia", title),
                None => match path.file_stem() {
                    Some(stem) => format!("{} - Ellisia", stem),
                    None => "Ellisia".into(),
                },
            };

            let config = json!({
                "book": {
                    "id": id,
                    "path": path.as_str(),
                },
                "renderer": format!("http://127.0.0.1:{port}"),
            });

            let window = WindowBuilder::new(&app, id.clone(), url)
                .title(title)
                .min_inner_size(900.0, 800.0)
                .inner_size(1200.0, 900.0)
                .center()
                .focused(true)
                .initialization_script(&format!("const ELLISIA = {config};"))
                // .on_navigation(|_| false)
                // .on_web_resource_request(|request, response| {})
                .build()
                .context("Failed to create reader window")?;

            window.on_window_event(move |event| {
                if let WindowEvent::Destroyed = event {
                    let state = app.state::<AppState>();
                    state.close_book(&id);
                }
            });

            #[cfg(any(debug_assertions, feature = "devtools"))]
            window.open_devtools();

            Ok(())
        }
    }
}
