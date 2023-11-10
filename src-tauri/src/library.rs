use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::epub::EpubFile;
use crate::utils::get_config_dir_path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Most of the time we do both read and write (e.g. updating reading state),
/// so we don't use a RwLock as it's not a read-heavy load.
pub struct Library {
    books: HashMap<String, Book>,
}

impl Library {
    fn get_path() -> Result<PathBuf> {
        let dir = get_config_dir_path()?;
        Ok(dir.join("library.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::get_path()?;
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

    pub fn persist(&mut self) -> Result<()> {
        let path = Self::get_path()?;
        let file = File::create(path).context("Failed to open library.json")?;
        serde_json::to_writer_pretty(file, self).context("Failed to write library.json")?;
        Ok(())
    }

    pub fn books(&self) -> &HashMap<String, Book> {
        &self.books
    }

    pub fn books_mut(&mut self) -> &mut HashMap<String, Book> {
        &mut self.books
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub path: String,
    pub location: Option<String>,
    #[serde(with = "humantime_serde")]
    pub last_read_at: SystemTime,
    #[serde(default)]
    pub metadata: BookMetadata,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BookMetadata {
    pub unique_id: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
}

impl BookMetadata {
    pub fn new(epub: &EpubFile) -> Self {
        Self {
            unique_id: epub.rootfile().get_unique_id(),
            title: epub
                .rootfile()
                .package
                .metadata
                .title
                .first()
                .cloned()
                .or_else(|| epub.path().file_stem().map(str::to_owned)),
            author: epub.rootfile().package.metadata.creator.first().cloned(),
        }
    }
}
