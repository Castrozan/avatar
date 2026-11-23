use std::path::PathBuf;

use futures_util::future::BoxFuture;
use serde_json::Value;

use super::process::run;
use super::{AgentChannel, ChannelFailure, ChannelRelease, no_release};
use crate::speaking::{AgentChannelPlan, plan_from_status};

#[derive(Debug, Clone)]
pub struct AvatarAudioGate {
    program: PathBuf,
}

impl AvatarAudioGate {
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl AgentChannel for AvatarAudioGate {
    fn open(&self) -> BoxFuture<'_, Result<ChannelRelease, ChannelFailure>> {
        Box::pin(open_gate(self.program.clone()))
    }
}

async fn open_gate(program: PathBuf) -> Result<ChannelRelease, ChannelFailure> {
    let status: Value = serde_json::from_str(&run(&program, &["status"]).await?)
        .map_err(|error| ChannelFailure(error.to_string()))?;
    match plan_from_status(&status)? {
        AgentChannelPlan::OpenAsIs => Ok(no_release()),
        AgentChannelPlan::MutedUntilReleased => unmute_for_speech(program).await,
    }
}

async fn unmute_for_speech(program: PathBuf) -> Result<ChannelRelease, ChannelFailure> {
    run(&program, &["mute", "agent", "0"]).await?;
    Ok(Box::pin(async move {
        run(&program, &["mute", "agent", "1"]).await.map(drop)
    }))
}
