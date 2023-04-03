use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubTocNcx {
    // #[serde(rename(deserialize = "docTitle"))]
    // doc_title: String,
    #[serde(rename(deserialize = "navMap"))]
    nav_map: EpubTocNavMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubTocNavMap {
    #[serde(rename(deserialize = "navPoint"))]
    children: Vec<EpubTocNavPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubTocNavPoint {
    #[serde(rename(deserialize = "@id"))]
    id: String,
    #[serde(rename(deserialize = "@playOrder"))]
    play_order: String,
    #[serde(rename(deserialize = "navLabel"))]
    nav_label: EpubTocNavLabel,
    #[serde(rename(deserialize = "content"))]
    content: EpubTocNavPointContent,
    #[serde(rename(deserialize = "navPoint"), default)]
    children: Vec<EpubTocNavPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubTocNavLabel {
    #[serde(rename(deserialize = "text"))]
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubTocNavPointContent {
    #[serde(rename(deserialize = "@src"))]
    src: String,
}
