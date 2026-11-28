use std::io;

use avatar_live::client::provisioning::{AgentApi, provision, signed_url};
use avatar_live::client::{ClientError, LiveClient, SessionRequest};
use tokio::sync::mpsc;

use super::handle::{Change, Reconnection, Replacement, VoiceCommand};

#[derive(Clone)]
pub(crate) struct Connector {
    api: AgentApi,
    agent: String,
    brief: Option<String>,
}

type Opened = (LiveClient, Option<String>);

fn reconnection(step: Reconnection) -> VoiceCommand {
    VoiceCommand::Reconnect(step)
}

fn replacement_of(opened: Result<Opened, ClientError>, change: Change) -> VoiceCommand {
    match opened {
        Ok((client, _)) => reconnection(Reconnection::Replace(Replacement {
            client,
            done: change.done,
        })),
        Err(failure) => reconnection(Reconnection::Failed(failure.to_string())),
    }
}

impl Connector {
    pub(crate) async fn provisioned(
        endpoint: &str,
        api_key: &str,
        brief: Option<String>,
    ) -> io::Result<Self> {
        let api = AgentApi::new(endpoint, api_key);
        let agent = provision(&api).await.map_err(io::Error::other)?;
        Ok(Self { api, agent, brief })
    }

    pub(crate) async fn connect(
        &self,
        speaking_voice: &str,
        taking_over: bool,
    ) -> Result<Opened, ClientError> {
        let url = signed_url(&self.api, &self.agent).await?;
        let request = SessionRequest {
            url: &url,
            speaking_voice,
            taking_over,
            brief: self.brief.as_deref(),
        };
        LiveClient::connect(&request).await
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
