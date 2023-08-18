use anyhow::Context;
use serde::Serialize;
use tauri::api::dialog;
use tauri::{AppHandle, Manager, Wry};
use typed_path::Utf8NativePathBuf;

use crate::epub::rootfile::EpubRootfile;
use crate::epub::toc::EpubToc;
use crate::error::CommandError;
use crate::library::{Book, BookMetadata};
use crate::state::AppState;
use crate::utils::now_unix_timestamp;

/// This handler creating new windows needs to be `async` to avoid deadlocks.
/// See https://tauri.app/v1/guides/features/multiwindow/#create-a-window-using-an-apphandle-instance
/// See https://github.com/tauri-apps/wry/issues/583
#[tauri::command]
pub async fn open_book(app: AppHandle, path: &str) -> Result<bool, CommandError> {
    let path = Utf8NativePathBuf::from(path);
    let result = crate::launch_book(app.clone(), path);

    if let Err(e) = &result {
        let library = app.get_window("library");
        let message = format!("{:?}", e);
        dialog::message::<Wry>(library.as_ref(), "Error", message);
    }

    Ok(result.is_ok())
}

#[tauri::command]
pub async fn open_library(app: AppHandle) -> Result<bool, CommandError> {
    let result = crate::launch_library(app);
    Ok(result.is_ok())
}

#[tauri::command]
pub async fn close_library(app: AppHandle) -> Result<(), CommandError> {
    if let Some(window) = app.get_window("library") {
        window.close().context("Failed to close library window.")?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_library(app: AppHandle) -> Result<impl Serialize, CommandError> {
    let state = app.state::<AppState>();
    let library = state.library().lock().clone();
    Ok(library)
}

#[tauri::command]
pub fn get_toc(app: AppHandle, id: &str) -> Result<EpubToc, CommandError> {
    let state = app.state::<AppState>();
    let epub = state.epubs().get(id).context("Book not opened")?;
    let toc = epub.toc().clone();
    Ok(toc)
}

#[tauri::command]
pub fn get_rootfile(app: AppHandle, id: &str) -> Result<EpubRootfile, CommandError> {
    let state = app.state::<AppState>();
    let epub = state.epubs().get(id).context("Book not opened")?;
    let toc = epub.rootfile().clone();
    Ok(toc)
}

#[tauri::command]
pub fn get_progress(app: AppHandle, id: &str) -> Result<(String, f64), CommandError> {
    let state = app.state::<AppState>();
    let mut library = state.library().lock();

    let (path, progress) = match library.books_mut().get(id) {
        Some(book) => (book.content_path.clone(), book.content_progress),
        None => ("".to_string(), 0.0),
    };

    Ok((path, progress))
}

#[tauri::command]
pub fn save_progress(
    app: AppHandle,
    id: &str,
    path: &str,
    progress: f64,
) -> Result<(), CommandError> {
    let state = app.state::<AppState>();
    let mut library = state.library().lock();

    match library.books_mut().get_mut(id) {
        Some(book) => {
            book.content_path = path.to_string();
            book.content_progress = progress;
            book.last_read_at = now_unix_timestamp();
        }
        None => {
            // Should not happen, but anyway we just create it.
            let epub = state.epubs().get(id).context("Book not opened")?;
            library.books_mut().insert(
                id.to_string(),
                Book {
                    path: epub.path().to_string(),
                    content_path: path.to_string(),
                    content_progress: progress,
                    last_read_at: now_unix_timestamp(),
                    metadata: BookMetadata::new(&epub),
                },
            );
        }
    }

    library.persist()?;

    Ok(())
}
