use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::Path;

use avatar_transcript::json::{self, Json, JsonObject};

const PRIVATE_DIRECTORY_MODE: u32 = 0o700;
const PRIVATE_FILE_MODE: u32 = 0o600;

pub(super) fn profile_text(embedding: &[f64], samples: i64) -> String {
    let mut object = JsonObject::default();
    let values = embedding.iter().copied().map(Json::Float).collect();
    object.set("embedding", Json::Array(values));
    object.set("samples", Json::Integer(samples));
    json::encode(&Json::Object(object))
}

fn tolerate_existing(outcome: io::Result<()>) -> io::Result<()> {
    match outcome {
        Err(error) if error.kind() != ErrorKind::AlreadyExists => Err(error),
        _ => Ok(()),
    }
}

fn create_private_directory(directory: &Path) -> io::Result<()> {
    if let Some(parent) = directory.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut builder = DirBuilder::new();
    tolerate_existing(builder.mode(PRIVATE_DIRECTORY_MODE).create(directory))
}

fn write_private(path: &Path, text: &str) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    options
        .mode(PRIVATE_FILE_MODE)
        .open(path)?
        .write_all(text.as_bytes())
}

pub(super) fn write_profile(directory: &Path, path: &Path, text: &str) -> io::Result<()> {
    create_private_directory(directory)?;
    let temporary = path.with_extension("tmp");
    write_private(&temporary, text)?;
    fs::rename(temporary, path)
}
