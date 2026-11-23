use std::sync::Arc;

use avatar_listening::barge_in::decision::{BargeIn, Reply};
use avatar_listening::barge_in::reply_link::control_socket::ReplyLink;
use avatar_listening::listener::capture::{BargeInHearing, NoBargeIn};
use avatar_listening::listener::onnx::SileroVoiceActivity;
use tokio::runtime::Handle;

pub(super) struct ReplyHandle {
    link: Arc<ReplyLink>,
    runtime: Handle,
}

impl ReplyHandle {
    pub(super) fn new(link: Arc<ReplyLink>, runtime: Handle) -> Self {
        Self { link, runtime }
    }
}

impl Reply for ReplyHandle {
    fn speaking(&self) -> bool {
        self.link.speaking()
    }

    fn stop(&mut self) {
        self.runtime.block_on(self.link.stop());
    }
}

pub(super) enum Hearing {
    Silent(NoBargeIn),
    Barging(Box<BargeIn<ReplyHandle, SileroVoiceActivity>>),
}

impl BargeInHearing for Hearing {
    fn begin_segment(&mut self) {
        match self {
            Self::Silent(silent) => silent.begin_segment(),
            Self::Barging(barging) => BargeInHearing::begin_segment(barging.as_mut()),
        }
    }

    fn hear(&mut self, frame: &[u8], loud: bool) {
        match self {
            Self::Silent(silent) => silent.hear(frame, loud),
            Self::Barging(barging) => BargeInHearing::hear(barging.as_mut(), frame, loud),
        }
    }
}
