use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::utils::get_config_dir;

#[derive(Debug, Default, Serialize, Deserialize)]
/// Most of the time we do both read and write (e.g. updating reading state),
/// so we don't use a RwLock as it's not a read-heavy load.
pub struct Library {
    books: HashMap<String, Book>,
}

impl Library {
    fn init_path() -> Result<PathBuf> {
        let dir = get_config_dir().context("Failed to get app config dir")?;
        if dir.is_file() {
            std::fs::remove_file(&dir).context("Failed to init app config dir")?;
        }
        if !dir.exists() {
            std::fs::create_dir_all(&dir).context("Failed to init app config dir")?;
        }
        Ok(dir.join("library.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::init_path()?;
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .context("Failed to open library.json")?;
        let reader = BufReader::new(file);
        let library = serde_json::from_reader(reader).unwrap_or_default();

        Ok(library)
    }

    pub fn books(&self) -> &HashMap<String, Book> {
        &self.books
    }

    pub fn books_mut(&mut self) -> &mut HashMap<String, Book> {
        &mut self.books
    }

    pub fn persist(&mut self) -> Result<()> {
        let path = Self::init_path()?;
        let file = File::create(path).context("Failed to open library.json")?;
        serde_json::to_writer_pretty(file, self).context("Failed to write library.json")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub path: String,
    pub hash: String,
    pub content_path: String,
    pub content_progress: f64,
    pub last_read_at: u64,
}
