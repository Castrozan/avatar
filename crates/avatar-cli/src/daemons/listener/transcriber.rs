mod settings;

#[cfg(test)]
mod scripted_sender;
#[cfg(test)]
mod tests;

use std::io;

use avatar_assets::ProcessEnvironment;
use avatar_listening::listener::daemon::{TranscribeFailure, Transcriber};
use avatar_listening::listener::diagnostics::Diagnostics;
use avatar_transcription::{
    HexClient, HttpSender, ReqwestSender, ScribeClient, TranscriptionError,
    transcribe_with_local_fallback,
};
use tokio::runtime::Handle;

use super::notes::StandardErrorNotes;
use crate::support::hex::discovery_file;
use settings::scribe_client;

pub(super) struct CloudFirstTranscriber<Sender> {
    runtime: Handle,
    cloud: ScribeClient<Sender>,
    local: HexClient<Sender>,
}

impl<Sender: HttpSender> CloudFirstTranscriber<Sender> {
    fn new(runtime: Handle, cloud: ScribeClient<Sender>, local: HexClient<Sender>) -> Self {
        Self {
            runtime,
            cloud,
            local,
        }
    }

    fn heard(
        &mut self,
        segment: &[u8],
        language: Option<&str>,
        diagnostics: &mut Vec<String>,
    ) -> Result<String, TranscriptionError> {
        let heard = transcribe_with_local_fallback(
            &mut self.cloud,
            &self.local,
            segment,
            language,
            diagnostics,
        );
        self.runtime.block_on(heard)
    }
}

impl CloudFirstTranscriber<ReqwestSender> {
    pub(super) fn from_environment(runtime: Handle) -> io::Result<Self> {
        let sender = ReqwestSender::default();
        let cloud = scribe_client(sender.clone(), &ProcessEnvironment);
        let local = HexClient::new(sender, discovery_file()?);
        Ok(Self::new(runtime, cloud, local))
    }
}

fn report(diagnostics: &[String]) {
    let mut notes = StandardErrorNotes;
    for line in diagnostics {
        notes.note(line);
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

impl<Sender: HttpSender> Transcriber for CloudFirstTranscriber<Sender> {
    fn transcribe(
        &mut self,
        segment: &[u8],
        language: Option<&str>,
    ) -> Result<String, TranscribeFailure> {
        let mut diagnostics = Vec::new();
        let outcome = self.heard(segment, language, &mut diagnostics);
        report(&diagnostics);
        outcome.map_err(|error| failure_of(&error))
    }
}
