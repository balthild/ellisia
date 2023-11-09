use std::collections::HashMap;
use std::io::BufReader;

use anyhow::{bail, Context, Result};
use serde::de::DeserializeOwned;
use typed_path::{Utf8NativePath, Utf8NativePathBuf};

use crate::zip::SharedZip;

use self::container::EpubContainer;
use self::rootfile::EpubRootfile;
use self::toc::EpubToc;

pub mod container;
pub mod rootfile;
pub mod toc;

#[derive(Debug)]
pub struct EpubFile {
    path: Utf8NativePathBuf,
    zip: SharedZip,
    container: EpubContainer,
    rootfile: EpubRootfile,
    toc: EpubToc,
    media_types: HashMap<String, String>,
}

impl EpubFile {
    pub fn open(path: Utf8NativePathBuf) -> Result<Self> {
        let zip = SharedZip::open(path.as_str())?;

        let container = read_container(&zip).context("Invalid EPUB file")?;
        let rootfile = read_rootfile(&zip, &container).context("Invalid EPUB file")?;

        let mut media_types = HashMap::new();
        for item in &rootfile.package.manifest.children {
            let path = rootfile.resolve_href(&item.href);
            media_types.insert(path, item.media_type.clone());
        }

        let toc = match &*rootfile.package.version {
            "2.0" => read_toc_ncx(&zip, &rootfile)?,
            // TODO: EPUB 3.0 new TOC
            "3.0" => read_toc_ncx(&zip, &rootfile)?,
            x => bail!("Unsupported EPUB version: {x}"),
        };

        Ok(Self {
            path,
            zip,
            container,
            rootfile,
            toc,
            media_types,
        })
    }

    pub fn path(&self) -> &Utf8NativePath {
        &self.path
    }

    pub fn container(&self) -> &EpubContainer {
        &self.container
    }

    pub fn rootfile(&self) -> &EpubRootfile {
        &self.rootfile
    }

    pub fn toc(&self) -> &EpubToc {
        &self.toc
    }

    pub fn get_media_type(&self, path: &str) -> Option<&str> {
        self.media_types.get(path).map(String::as_str)
    }

    pub fn read_file(&mut self, path: &str) -> Result<Vec<u8>> {
        self.zip.read(path)
    }
}

pub fn read_xml<T: DeserializeOwned>(zip: &SharedZip, path: &str) -> Result<T> {
    let reader = BufReader::new(zip.by_name(path)?);
    quick_xml::de::from_reader(reader).with_context(|| format!("Failed to parse {path} as XML"))
}

fn read_container(zip: &SharedZip) -> Result<EpubContainer> {
    read_xml(zip, "META-INF/container.xml")
}

fn read_rootfile(zip: &SharedZip, container: &EpubContainer) -> Result<EpubRootfile> {
    let path = container.rootfiles.children[0].full_path.clone();
    let package = read_xml(zip, &path)?;
    Ok(EpubRootfile::new(path, package))
}

fn read_toc_ncx(zip: &SharedZip, rootfile: &EpubRootfile) -> Result<EpubToc> {
    let href = &rootfile
        .package
        .manifest
        .children
        .iter()
        .find(|x| x.id == rootfile.package.spine.toc)
        .context("Failed to find ToC file in manifest")?
        .href;

    let path = rootfile.resolve_href(href);
    let ncx = read_xml(zip, &path)?;

    Ok(EpubToc::new(path, ncx))
}
