use std::io;

use avatar_listening::listener::daemon::{TranscribeFailure, Transcriber};
use avatar_transcription::{HexClient, ReqwestSender, TranscriptionError};
use tokio::runtime::Handle;

use crate::support::hex::discovery_file;

pub(super) struct HexTranscriber {
    runtime: Handle,
    client: HexClient<ReqwestSender>,
}

impl HexTranscriber {
    pub(super) fn new(runtime: Handle) -> io::Result<Self> {
        let client = HexClient::new(ReqwestSender::default(), discovery_file()?);
        Ok(Self { runtime, client })
    }
}

fn failure_of(error: &TranscriptionError) -> TranscribeFailure {
    let text = error.to_string();
    match error {
        TranscriptionError::MalformedBody(_) | TranscriptionError::MissingField(_) => {
            TranscribeFailure::Failed(text)
        }
        _ => TranscribeFailure::Unreachable(text),
    }
}

impl Transcriber for HexTranscriber {
    fn transcribe(
        &mut self,
        segment: &[u8],
        language: Option<&str>,
    ) -> Result<String, TranscribeFailure> {
        let heard = self.client.transcribe(segment, language);
        self.runtime
            .block_on(heard)
            .map_err(|error| failure_of(&error))
    }
}
