use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EpubContainer {
    pub rootfiles: EpubContainerRootfiles,
}

#[derive(Debug, Deserialize)]
pub struct EpubContainerRootfiles {
    #[serde(rename(deserialize = "$value"))]
    pub children: Vec<EpubContainerRootfile>,
}

#[derive(Debug, Deserialize)]
pub struct EpubContainerRootfile {
    #[serde(rename(deserialize = "@full-path"))]
    pub full_path: String,
    #[serde(rename(deserialize = "@media-type"))]
    pub media_type: String,
}
