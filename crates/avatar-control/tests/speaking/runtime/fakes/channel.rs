use std::collections::VecDeque;
use std::sync::Arc;

use avatar_control::speaking::runtime::{AgentChannel, ChannelFailure, ChannelRelease};
use futures_util::future::BoxFuture;
use serde_json::json;

use crate::runtime::script::Script;
use crate::runtime::shared::{Gate, Gates, Shared, Waiting, new_gate, ready_waiting};

pub(crate) struct FakeChannel {
    pub(crate) shared: Shared,
    pub(crate) script: Arc<Script>,
}

fn hold_queue(
    shared: &Shared,
    held: bool,
    queue: fn(&mut Gates) -> &mut VecDeque<Gate>,
) -> Waiting {
    if !held {
        return ready_waiting();
    }
    let (gate, waiting) = new_gate();
    shared.with(|recording| queue(&mut recording.gates).push_back(gate));
    waiting
}

async fn release_channel(shared: Shared, script: Arc<Script>) -> Result<(), ChannelFailure> {
    shared.record(json!({"channel": "release"}));
    hold_queue(&shared, script.holds("release"), |gates| &mut gates.release).await;
    script
        .faults
        .release
        .clone()
        .map_or(Ok(()), |message| Err(ChannelFailure(message)))
}

async fn open_channel(
    shared: Shared,
    script: Arc<Script>,
) -> Result<ChannelRelease, ChannelFailure> {
    hold_queue(&shared, script.holds("open"), |gates| &mut gates.open).await;
    match script.faults.open.clone() {
        Some(message) => Err(ChannelFailure(message)),
        None => Ok(Box::pin(release_channel(shared, script))),
    }
}

impl AgentChannel for FakeChannel {
    fn open(&self) -> BoxFuture<'_, Result<ChannelRelease, ChannelFailure>> {
        self.shared.record(json!({"channel": "open"}));
        Box::pin(open_channel(self.shared.clone(), Arc::clone(&self.script)))
    }
}
