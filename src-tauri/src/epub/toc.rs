use serde::Serialize;

use ncx::EpubTocNcx;

pub mod ncx;

#[derive(Debug, Clone, Serialize)]
pub struct EpubToc {
    pub path: String,
    pub ncx: EpubTocNcx,
}

impl EpubToc {
    pub fn new(path: String, ncx: EpubTocNcx) -> Self {
        Self { path, ncx }
    }
}
