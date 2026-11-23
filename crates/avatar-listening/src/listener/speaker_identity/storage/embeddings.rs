use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use avatar_transcript::json::{self, Json, JsonObject};
use serde::Deserialize;

use crate::listener::speaker_identity::error::VoiceError;

const PRIVATE_FILE_MODE: u32 = 0o600;

#[derive(Deserialize)]
struct EmbeddingEntry {
    sequence: i64,
    embedding: Vec<f64>,
}

fn entry_line(sequence: i64, embedding: &[f64]) -> String {
    let mut object = JsonObject::default();
    object.set("sequence", Json::Integer(sequence));
    let values = embedding.iter().copied().map(Json::Float).collect();
    object.set("embedding", Json::Array(values));
    format!("{}\n", json::encode(&Json::Object(object)))
}

pub fn append_embedding(path: &Path, sequence: i64, embedding: &[f64]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .mode(PRIVATE_FILE_MODE)
        .open(path)?;
    file.write_all(entry_line(sequence, embedding).as_bytes())
}

fn read_optional(path: &Path) -> Result<String, VoiceError> {
    match fs::read_to_string(path) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(String::new()),
        other => Ok(other?),
    }
}

pub fn read_embeddings(path: &Path, first: i64, last: i64) -> Result<Vec<Vec<f64>>, VoiceError> {
    let mut chosen = Vec::new();
    for line in read_optional(path)?.lines() {
        let entry: EmbeddingEntry = serde_json::from_str(line)?;
        (first <= entry.sequence && entry.sequence <= last).then(|| chosen.push(entry.embedding));
    }
    Ok(chosen)
}
