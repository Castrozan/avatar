use std::io;

use serde::Deserialize;

const EMBEDDED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../runtime/assets/manifest.json"
));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ArchiveKind {
    #[serde(rename = "tar.gz")]
    TarGzip,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Download {
    pub name: String,
    pub url: String,
    pub integrity: String,
    pub destination: String,
    #[serde(default)]
    pub archive: Option<ArchiveKind>,
}

#[derive(Deserialize)]
struct Manifest {
    downloads: Vec<Download>,
}

pub fn parse_downloads(text: &str) -> io::Result<Vec<Download>> {
    serde_json::from_str::<Manifest>(text)
        .map(|manifest| manifest.downloads)
        .map_err(io::Error::other)
}

pub fn embedded_downloads() -> io::Result<Vec<Download>> {
    parse_downloads(EMBEDDED)
}

#[cfg(test)]
mod tests;
