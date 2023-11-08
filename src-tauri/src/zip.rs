use std::fs::File;
use std::io::Read;

use anyhow::{Context, Result};
use crossbeam_channel::{Receiver, RecvError, SendError, Sender};
use zip::ZipArchive;

#[derive(Debug, Clone)]
pub struct ZipPool {
    push: Sender<ZipArchive<File>>,
    pull: Receiver<ZipArchive<File>>,
}

impl ZipPool {
    pub fn open(path: &str) -> Result<Self> {
        let cap = num_cpus::get();
        let (push, pull) = crossbeam_channel::bounded(cap);

        for _ in 0..cap {
            let file = File::open(path).context(format!("Failed to open file: {path}"))?;
            let zip = ZipArchive::new(file).context("Failed to read zip archive")?;
            push.send(zip)?;
        }

        Ok(Self { push, pull })
    }

    pub fn read(&self, path: &str) -> Result<Vec<u8>> {
        let mut zip = self.pull()?;
        let result = read_bytes(&mut zip, path);
        self.push(zip)?;
        result
    }

    fn pull(&self) -> Result<ZipArchive<File>, RecvError> {
        self.pull.recv()
    }

    fn push(&self, zip: ZipArchive<File>) -> Result<(), SendError<ZipArchive<File>>> {
        self.push.send(zip)
    }
}

pub fn read_bytes(zip: &mut ZipArchive<File>, path: &str) -> Result<Vec<u8>> {
    match zip.by_name(path) {
        Ok(mut file) => {
            let mut bytes = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut bytes)?;
            Ok(bytes)
        }
        Err(e) => Err(e).with_context(|| format!("failed to read {path}")),
    }
}
