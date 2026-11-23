use std::io;
use std::path::PathBuf;

use avatar_transcription::hex_discovery_file;

pub(crate) fn discovery_file() -> io::Result<PathBuf> {
    let home = std::env::var_os("HOME").ok_or_else(|| io::Error::other("HOME is not set"))?;
    let platform = if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    };
    Ok(hex_discovery_file(&PathBuf::from(home), platform))
}
