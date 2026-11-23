use std::path::PathBuf;

use avatar_transcription::{HexClient, ReqwestSender, hex_discovery_file};
use tokio::runtime::Runtime;

fn discovery_file() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    let platform = if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    };
    hex_discovery_file(&home, platform)
}

pub(crate) fn transcribed(runtime: &Runtime, segment: &[u8]) -> Result<String, String> {
    let client = HexClient::new(ReqwestSender::default(), discovery_file());
    let spoken = runtime.block_on(client.transcribe(segment, None));
    spoken.map_err(|error| error.to_string())
}
