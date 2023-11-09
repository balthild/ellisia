use serde::{Deserialize, Serialize};
use typed_path::Utf8UnixPathBuf;

use crate::utils::clean_path_unix;

#[derive(Debug, Clone, Serialize)]
pub struct EpubRootfile {
    pub path: String,
    pub package: EpubRootfilePackage,
}

impl EpubRootfile {
    pub fn new(path: String, package: EpubRootfilePackage) -> Self {
        Self { path, package }
    }

    pub fn resolve_href(&self, href: &str) -> String {
        let mut path = Utf8UnixPathBuf::from(&self.path);
        // `self.path` is the rootfile. Remove the filename to get the base dir.
        path.pop();
        path.push(href);
        clean_path_unix(&path).to_string()
    }

    pub fn get_unique_id(&self) -> Option<String> {
        self.package
            .metadata
            .identifier
            .iter()
            .find(|ident| ident.id.as_deref() == Some(&self.package.unique_identifier))
            .map(|ident| ident.value.clone())
    }

    pub fn get_cover_path(&self) -> Option<String> {
        let id = self
            .package
            .metadata
            .meta
            .iter()
            .find(|meta| meta.name.as_deref() == Some("cover"))
            .and_then(|meta| meta.content.as_deref())?;

        let href = self
            .package
            .manifest
            .children
            .iter()
            .find(|item| item.id == id)
            .map(|item| &*item.href)?;

        Some(self.resolve_href(href))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubRootfilePackage {
    #[serde(rename(deserialize = "@unique-identifier"))]
    pub unique_identifier: String,
    #[serde(rename(deserialize = "@version"))]
    pub version: String,
    #[serde(rename(deserialize = "metadata"))]
    pub metadata: EpubRootfileMetadata,
    #[serde(rename(deserialize = "manifest"))]
    pub manifest: EpubRootfileManifest,
    #[serde(rename(deserialize = "spine"))]
    pub spine: EpubRootfileSpine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubRootfileMetadata {
    #[serde(rename(deserialize = "identifier"))]
    pub identifier: Vec<EpubRootfileMetadataIdentifier>,
    #[serde(rename(deserialize = "title"))]
    pub title: Vec<String>,
    #[serde(rename(deserialize = "creator"), default)]
    pub creator: Vec<String>,
    #[serde(rename(deserialize = "publisher"), default)]
    pub publisher: Vec<String>,
    #[serde(rename(deserialize = "meta"))]
    pub meta: Vec<EpubRootfileMetadataMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubRootfileMetadataIdentifier {
    #[serde(rename(deserialize = "@id"))]
    pub id: Option<String>,
    #[serde(rename(deserialize = "@scheme"))]
    pub scheme: Option<String>,
    #[serde(rename(deserialize = "$value"))]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubRootfileMetadataMeta {
    #[serde(rename(deserialize = "@name"))]
    pub name: Option<String>,
    #[serde(rename(deserialize = "@content"))]
    pub content: Option<String>,
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
