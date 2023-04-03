use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct EpubRootfile {
    pub path: String,
    // pub hash: Vec<u8>,
    pub package: EpubRootfilePackage,
}

impl EpubRootfile {
    pub fn new(path: String, package: EpubRootfilePackage) -> Self {
        Self { path, package }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubRootfilePackage {
    #[serde(rename(deserialize = "@version"))]
    pub version: String,
    #[serde(rename(deserialize = "manifest"))]
    pub manifest: EpubRootfileManifest,
    #[serde(rename(deserialize = "spine"))]
    pub spine: EpubRootfileSpine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubRootfileManifest {
    #[serde(rename(deserialize = "item"))]
    pub children: Vec<EpubRootfileManifestItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubRootfileManifestItem {
    #[serde(rename(deserialize = "@id"))]
    pub id: String,
    #[serde(rename(deserialize = "@href"))]
    pub href: String,
    #[serde(rename(deserialize = "@media-type"))]
    pub media_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubRootfileSpine {
    #[serde(rename(deserialize = "@toc"))]
    pub toc: String,
    #[serde(rename(deserialize = "itemref"))]
    pub children: Vec<EpubRootfileSpineItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubRootfileSpineItem {
    #[serde(rename(deserialize = "@idref"))]
    pub idref: String,
    #[serde(rename(deserialize = "@properties"))]
    pub properties: Option<String>,
}
