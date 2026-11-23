use crate::error::{Outcome, TranscriptionError, missing_field, python_display};
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

const PREFERRED_MODEL: &str = "parakeet_v3";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(super) struct HexDiscovery {
    pub(super) port: u16,
    pub(super) token: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct HexModel {
    pub id: String,
    pub installed: bool,
}

#[must_use]
pub fn hex_discovery_file(home: &Path, platform: &str) -> PathBuf {
    let directory = match platform {
        "darwin" => "Library/Application Support/hex",
        _ => ".local/share/hex",
    };
    home.join(directory).join("local-api.json")
}

#[must_use]
pub fn transcription_path(model: &str, language: Option<&str>) -> String {
    let mut query = format!("model={}", quote_plus(model));
    if let Some(named) = language.filter(|code| !code.is_empty()) {
        query = format!("{query}&language={}", quote_plus(named));
    }
    format!("/transcriptions?{query}")
}

fn quote_plus(text: &str) -> String {
    text.bytes().map(quote_byte).collect()
}

fn quote_byte(byte: u8) -> String {
    match byte {
        b' ' => "+".to_owned(),
        _ if is_unreserved(byte) => char::from(byte).to_string(),
        _ => format!("%{byte:02X}"),
    }
}

fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-' | b'~')
}

pub fn installed_model(models: &[HexModel]) -> Outcome<&str> {
    let preferred = models
        .iter()
        .find(|model| model.installed && model.id == PREFERRED_MODEL);
    let first_installed = models.iter().find(|model| model.installed);
    preferred
        .or(first_installed)
        .map(|model| model.id.as_str())
        .ok_or(TranscriptionError::NoModelInstalled)
}

pub(super) fn refusal_code_text(answer: &Value) -> String {
    python_display(answer.get("code"))
}

pub(super) fn discovery_of(value: &Value) -> Outcome<HexDiscovery> {
    Ok(HexDiscovery {
        port: port_of(value)?,
        token: token_of(value)?.to_owned(),
    })
}

fn port_of(value: &Value) -> Outcome<u16> {
    value
        .get("port")
        .and_then(Value::as_u64)
        .and_then(|port| u16::try_from(port).ok())
        .ok_or_else(|| missing_field("port"))
}

fn token_of(value: &Value) -> Outcome<&str> {
    value
        .get("token")
        .and_then(Value::as_str)
        .ok_or_else(|| missing_field("token"))
}
