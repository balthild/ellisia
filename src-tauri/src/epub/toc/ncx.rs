use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubTocNcx {
    #[serde(rename(deserialize = "docTitle"))]
    doc_title: EpubTocNcxDocTitle,
    #[serde(rename(deserialize = "navMap"))]
    nav_map: EpubTocNcxNavMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubTocNcxDocTitle {
    #[serde(rename(deserialize = "text"))]
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubTocNcxNavMap {
    #[serde(rename(deserialize = "navPoint"))]
    children: Vec<EpubTocNcxNavPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubTocNcxNavPoint {
    #[serde(rename(deserialize = "@id"))]
    id: String,
    #[serde(rename(deserialize = "@playOrder"))]
    play_order: String,
    #[serde(rename(deserialize = "navLabel"))]
    nav_label: EpubTocNcxNavLabel,
    #[serde(rename(deserialize = "content"))]
    content: EpubTocNcxNavPointContent,
    #[serde(rename(deserialize = "navPoint"), default)]
    children: Vec<EpubTocNcxNavPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubTocNcxNavLabel {
    #[serde(rename(deserialize = "text"))]
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubTocNcxNavPointContent {
    #[serde(rename(deserialize = "@src"))]
    src: String,
}
