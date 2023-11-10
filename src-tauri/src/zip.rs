use std::error::Error;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::Read;

use anyhow::{bail, Context, Result};
use positioned_io::{Cursor, RandomAccessFile};
use rc_zip::reader::sync::EntryReader;
use rc_zip::reader::{ArchiveReader, ArchiveReaderResult};
use rc_zip::{Archive, EntryContents, StoredEntry};
use typed_path::Utf8UnixPathBuf;

use crate::path::Utf8PathExtClean;

#[derive(Debug)]
pub enum ZipError {
    EntryNotFound,
    EntryIsNotFile,
}

impl Display for ZipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZipError::EntryNotFound => f.write_str("The zip entry not found"),
            ZipError::EntryIsNotFile => f.write_str("Te zip entry is not a file"),
        }
    }
}

impl Error for ZipError {}

pub struct SharedZip {
    file: RandomAccessFile,
    archive: Archive,
}

impl SharedZip {
    pub fn open(path: &str) -> Result<Self> {
        let file = File::open(path).with_context(|| format!("Failed to open file: {path}"))?;
        let size = file.metadata()?.len();

        let file = RandomAccessFile::try_new(file)?;

        // Copied and modified from the `rc-zip` crate.
        let mut reader = ArchiveReader::new(size);
        loop {
            if let Some(offset) = reader.wants_read() {
                let mut cursor = Cursor::new_pos(&file, offset);
                match reader.read(&mut cursor) {
                    Ok(read_bytes) => {
                        if read_bytes == 0 {
                            bail!("Unexpected EOF when processing zip file: {path}");
                        }
                    }
                    Err(e) => return Err(e).context(format!("Failed to read zip file: {path}")),
                }
            }

            match reader.process() {
                Ok(ArchiveReaderResult::Continue) => continue,
                Ok(ArchiveReaderResult::Done(archive)) => return Ok(Self { file, archive }),
                Err(e) => return Err(e).context(format!("Invalid zip file: {path}")),
            }
        }
    }

    pub fn entry(&self, path: &str) -> Result<SharedZipEntry, ZipError> {
        let path = Utf8UnixPathBuf::from(dbg!(path))
            .clean()
            .components()
            .skip_while(|component| {
                use typed_path::Utf8UnixComponent::*;
                matches!(component, ParentDir | CurDir | RootDir)
            })
            .collect::<Utf8UnixPathBuf>();

        let mut candidate = None;
        for entry in self.archive.entries() {
            if entry.name() == path {
                candidate = Some(SharedZipEntry::new(&self.file, entry));
                break;
            }

            if candidate.is_none() {
                let normalized = Utf8UnixPathBuf::from(entry.name()).clean();
                if normalized == path {
                    candidate = Some(SharedZipEntry::new(&self.file, entry));
                }
            }
        }

        match candidate {
            Some(entry) => match entry.entry.contents() {
                EntryContents::File => Ok(entry),
                _ => Err(ZipError::EntryIsNotFile),
            },
            None => Err(ZipError::EntryNotFound),
        }
    }
}

impl Debug for SharedZip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedZip").finish()
    }
}

pub struct SharedZipEntry<'a> {
    file: &'a RandomAccessFile,
    entry: &'a StoredEntry,
}

impl<'a> SharedZipEntry<'a> {
    fn new(file: &'a RandomAccessFile, entry: &'a StoredEntry) -> Self {
        Self { file, entry }
    }

    pub fn reader(&self) -> EntryReader<'_, Cursor<&RandomAccessFile>> {
        EntryReader::new(self.entry, |offset| Cursor::new_pos(self.file, offset))
    }

    pub fn bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.reader().read_to_end(&mut buf)?;
        Ok(buf)
    }
}
