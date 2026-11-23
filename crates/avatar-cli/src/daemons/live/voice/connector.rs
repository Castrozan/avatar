use std::path::{Path, PathBuf};

use avatar_live::client::{ClientError, GptLiveClient, SessionRequest};
use tokio::sync::mpsc;

use super::handle::{Change, Reconnection, Replacement, VoiceCommand};

#[derive(Clone)]
pub(crate) struct Connector {
    endpoint: String,
    key_file: PathBuf,
}

type Opened = (GptLiveClient, Option<String>);

fn reconnection(step: Reconnection) -> VoiceCommand {
    VoiceCommand::Reconnect(step)
}

fn replacement_of(opened: Result<Opened, ClientError>, change: Change) -> VoiceCommand {
    match opened {
        Ok((client, _)) => reconnection(Reconnection::Replace(Replacement {
            client,
            voice: change.voice,
            done: change.done,
        })),
        Err(failure) => reconnection(Reconnection::Failed(failure.to_string())),
    }
}

impl Connector {
    pub(crate) fn new(endpoint: &str, key_file: &Path) -> Self {
        Self {
            endpoint: endpoint.to_owned(),
            key_file: key_file.to_path_buf(),
        }
    }

    async fn key(&self) -> Result<String, ClientError> {
        let text = tokio::fs::read_to_string(&self.key_file).await;
        text.map_err(|failure| ClientError(failure.to_string()))
    }

    pub(crate) async fn connect(
        &self,
        speaking_voice: &str,
        taking_over: bool,
    ) -> Result<Opened, ClientError> {
        let key = self.key().await?;
        let request = self.request(&key, speaking_voice, taking_over);
        GptLiveClient::connect(&request).await
    }

    fn request<'a>(
        &'a self,
        key: &'a str,
        voice: &'a str,
        taking_over: bool,
    ) -> SessionRequest<'a> {
        SessionRequest {
            endpoint: &self.endpoint,
            api_key: key,
            speaking_voice: voice,
            taking_over,
        }
    }

    pub(crate) async fn replace(
        self,
        change: Change,
        commands: mpsc::UnboundedSender<VoiceCommand>,
    ) {
        let opened = self.connect(&change.voice, true).await;
        let _ = commands.send(replacement_of(opened, change));
    }
}
