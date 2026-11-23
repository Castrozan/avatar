use futures_util::future::BoxFuture;

use super::{AgentChannel, ChannelFailure, ChannelRelease, no_release};

#[derive(Debug, Default, Clone, Copy)]
pub struct SeatAgentChannel;

impl AgentChannel for SeatAgentChannel {
    fn open(&self) -> BoxFuture<'_, Result<ChannelRelease, ChannelFailure>> {
        Box::pin(std::future::ready(Ok(no_release())))
    }
}
