use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::time::SystemTime;

use anyhow::{bail, Context, Result};
use parking_lot::{Mutex, RwLock};
use rand::distributions::{Alphanumeric, DistString};
use typed_path::Utf8NativePathBuf;

use crate::epub::EpubFile;
use crate::library::{Book, BookMetadata, Library};

/// Most of the time we do both read and write (e.g. updating reading state),
/// so we don't use a `RwLock<Library>`. Also, we need to persist the entire
/// library to disk at every write, so we don't use locks on individual books.
pub struct AppState {
    renderer_port: u16,
    library: Mutex<Library>,
    epubs: RwLock<HashMap<String, EpubFile>>,
}

impl AppState {
    pub fn init(renderer_port: u16) -> Result<Self> {
        let library = Library::load().context("Failed to load library.json")?;

        Ok(Self {
            renderer_port,
            library: Mutex::new(library),
            epubs: RwLock::new(HashMap::new()),
        })
    }

    pub fn renderer_port(&self) -> u16 {
        self.renderer_port
    }

    pub fn library(&self) -> &Mutex<Library> {
        &self.library
    }

    pub fn epubs(&self) -> &RwLock<HashMap<String, EpubFile>> {
        &self.epubs
    }

    pub fn open_book(&self, path: Utf8NativePathBuf) -> Result<(String, Book)> {
        let mut library = self.library.lock();
        let mut epubs = self.epubs.write();

        let id = match library.books().iter().find(|(_, book)| book.path == path) {
            Some((id, _)) => id.clone(),
            None => {
                let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
                if library.books().contains_key(&id) {
                    bail!("You are too lucky! Please try again.");
                }
                id
            }
        };

        let epub = match epubs.entry(id.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let epub = EpubFile::open(path.clone()).context("Failed to open epub.")?;
                entry.insert(epub)
            }
        };

        let metadata = BookMetadata::new(epub);
        let book = match library.books_mut().entry(id.clone()) {
            Entry::Occupied(entry) => {
                let book = entry.into_mut();
                book.last_read_at = SystemTime::now();
                book.metadata = metadata;
                book
            }
            Entry::Vacant(entry) => {
                let book = Book {
                    path: path.to_string(),
                    location: None,
                    last_read_at: SystemTime::now(),
                    metadata: BookMetadata::new(epub),
                };
                entry.insert(book)
            }
        };

        Ok((id, book.clone()))
    }

    pub fn close_book(&self, id: &str) {
        let mut epubs = self.epubs.write();
        epubs.remove(id);
    }
}
