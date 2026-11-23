use std::io::{self, Read, Write};
use std::path::PathBuf;

use avatar_transcription::{HexClient, ReqwestSender, hex_discovery_file};
use tokio::runtime::Builder;

fn discovery_file() -> io::Result<PathBuf> {
    let home = std::env::var_os("HOME").ok_or_else(|| io::Error::other("HOME is not set"))?;
    let platform = if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    };
    Ok(hex_discovery_file(&PathBuf::from(home), platform))
}

pub(crate) fn run() -> io::Result<()> {
    let mut pcm = Vec::new();
    io::stdin().lock().read_to_end(&mut pcm)?;
    let client = HexClient::new(ReqwestSender::default(), discovery_file()?);
    let runtime = Builder::new_multi_thread().enable_all().build()?;
    let text = runtime
        .block_on(client.transcribe(&pcm, None))
        .map_err(|error| io::Error::other(error.to_string()))?;
    writeln!(io::stdout().lock(), "{text}")
}
