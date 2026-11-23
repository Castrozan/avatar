use std::sync::Arc;

use super::channel::AgentChannel;
use super::clock::Clock;
use super::platform::Platform;
use super::players::Players;

#[derive(Clone)]
pub struct SpeechServices {
    pub clock: Arc<dyn Clock>,
    pub channel: Arc<dyn AgentChannel>,
    pub players: Arc<dyn Players>,
    pub platform: Platform,
}
