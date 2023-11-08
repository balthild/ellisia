use std::time::SystemTime;

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

/// The handlers creating new windows need to be `async` to avoid deadlocks.
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
    let epubs = state.epubs().read();
    let epub = epubs.get(id).context("Book not opened")?;
    let toc = epub.toc().clone();
    Ok(toc)
}

#[tauri::command]
pub fn get_rootfile(app: AppHandle, id: &str) -> Result<EpubRootfile, CommandError> {
    let state = app.state::<AppState>();
    let epubs = state.epubs().read();
    let epub = epubs.get(id).context("Book not opened")?;
    let rootfile = epub.rootfile().clone();
    Ok(rootfile)
}

#[tauri::command]
pub fn get_progress(app: AppHandle, id: &str) -> Result<Option<String>, CommandError> {
    let state = app.state::<AppState>();
    let library = state.library().lock();
    let location = library.books().get(id).and_then(|x| x.location.to_owned());
    Ok(location)
}

#[tauri::command]
pub fn save_progress(app: AppHandle, id: &str, location: &str) -> Result<(), CommandError> {
    let state = app.state::<AppState>();
    let mut library = state.library().lock();

    match library.books_mut().get_mut(id) {
        Some(book) => {
            book.location = Some(location.to_string());
            book.last_read_at = SystemTime::now();
        }
        None => {
            // May happen when user removed book with the reader window opening.
            if let Some(epub) = state.epubs().read().get(id) {
                library.books_mut().insert(
                    id.to_string(),
                    Book {
                        path: epub.path().to_string(),
                        location: Some(location.to_string()),
                        last_read_at: SystemTime::now(),
                        metadata: BookMetadata::new(epub),
                    },
                );
            }
        }
    }

    library.persist()?;

    Ok(())
}
