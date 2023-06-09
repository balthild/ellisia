use anyhow::{Context, Result, bail};
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use parking_lot::Mutex;
use rand::distributions::{Alphanumeric, DistString};
use typed_path::Utf8NativePathBuf;

use crate::epub::EpubFile;
use crate::library::{Book, Library};
use crate::utils::{clean_path, now_unix_timestamp};

/// Most of the time we do both read and write (e.g. updating reading state),
/// so we don't use a `RwLock<Library>`. Also, we need to persist the entire
/// library to disk at every write, so we don't use locks on `Book` items.
pub struct AppState {
    renderer_port: u16,
    library: Mutex<Library>,
    epubs: DashMap<String, EpubFile>,
}

impl AppState {
    pub fn init(renderer_port: u16) -> Result<Self> {
        let library = Library::load().context("Failed to load library.json")?;

        Ok(Self {
            renderer_port,
            library: Mutex::new(library),
            epubs: DashMap::new(),
        })
    }

    pub fn renderer_port(&self) -> u16 {
        self.renderer_port
    }

    pub fn library(&self) -> &Mutex<Library> {
        &self.library
    }

    pub fn epubs(&self) -> &DashMap<String, EpubFile> {
        &self.epubs
    }

    pub fn open_book(&self, path: Utf8NativePathBuf) -> Result<String> {
        let path = clean_path(&path);
        // let id = base64_url::encode(path.as_str());

        // TODO: use uuid instead of base64(path)
        let mut library = self.library.lock();

        let book = library.books_mut().iter().find(|(_, book)| book.path == path);
        let id = match book {
            Some((id, _)) => id.clone(),
            None => {
                let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
                if library.books().contains_key(&id) {
                    bail!("You are too lucky!");
                }

                let book = Book {
                    path: path.to_string(),
                    hash: "".to_string(),
                    content_path: "".to_string(),
                    content_progress: 0.0,
                    last_read_at: now_unix_timestamp(),
                };
                library.books_mut().insert(id.clone(), book);

                id
            }
        };

        if let Entry::Vacant(entry) = self.epubs.entry(id.clone()) {
            let epub = EpubFile::open(path.clone()).context("Failed to open epub.")?;
            entry.insert(epub);
        }

        Ok(id)
    }
}
