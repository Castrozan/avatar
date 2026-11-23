use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use serde::Deserialize;

use crate::listener::session::CONFIGURATION_FILE;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ListenerConfiguration {
    pub sources: BTreeMap<String, String>,
    #[serde(default)]
    pub barge_in: bool,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub identify_speakers: bool,
}

pub fn read_configuration(directory: &Path) -> io::Result<ListenerConfiguration> {
    let text = std::fs::read_to_string(directory.join(CONFIGURATION_FILE))?;
    serde_json::from_str(&text).map_err(io::Error::other)
}
