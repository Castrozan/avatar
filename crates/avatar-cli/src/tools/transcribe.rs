use std::io::{self, Read, Write};

use avatar_transcription::{HexClient, ReqwestSender};
use tokio::runtime::Builder;

use crate::support::hex::discovery_file;

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
