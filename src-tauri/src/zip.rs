use std::error::Error;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::Read;

use anyhow::{bail, Context, Result};
use positioned_io::{Cursor, RandomAccessFile};
use rc_zip::reader::sync::EntryReader;
use rc_zip::reader::{ArchiveReader, ArchiveReaderResult};
use rc_zip::Archive;

#[derive(Debug)]
pub struct EntryNotFound;

impl Display for EntryNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Entry not found in the zip archive")
    }
}

impl Error for EntryNotFound {}

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
                Ok(ArchiveReaderResult::Continue) => {},
                Ok(ArchiveReaderResult::Done(archive)) => return Ok(Self { file, archive }),
                Err(e) => return Err(e).context(format!("Invalid zip file: {path}")),
            }
        }
    }

    pub fn by_name(&self, path: &str) -> Result<EntryReader<'_, Cursor<&RandomAccessFile>>> {
        let entry = self.archive.by_name(path).ok_or(EntryNotFound)?;
        let reader = EntryReader::new(entry, |offset| Cursor::new_pos(&self.file, offset));
        Ok(reader)
    }

    pub fn read(&self, path: &str) -> Result<Vec<u8>> {
        let mut reader = self.by_name(path)?;
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

impl Debug for SharedZip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedZip").finish()
    }
}
