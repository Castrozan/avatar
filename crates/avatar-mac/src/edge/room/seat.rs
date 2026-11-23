use avatar_seat::edges::{HttpProbe, SeatProbe};
use avatar_seat::local::SeatConfig;
use serde_json::Value;

use super::MacRoom;
use crate::client::{PORT, SeatEndpoint};
use crate::edge::client::SeatClient;
use crate::edge::stage::BrowserProcesses;
use crate::failure::Failure;
use crate::meeting::SeatHealth;
use crate::session::SeatControl;

const CONVERSATION_PROFILE: &str = ".local/share/avatar/conversation-browser";

pub struct MacSeatControl {
    pub client: SeatClient,
    pub processes: BrowserProcesses,
    pub profile_flag: String,
}

impl MacSeatControl {
    #[must_use]
    pub fn profile_flag_for(home: &str) -> String {
        format!("--user-data-dir={home}/{CONVERSATION_PROFILE}")
    }
}

impl SeatControl for MacRoom {
    fn seat_attached(&mut self) -> bool {
        self.seat.client.attached()
    }

    fn hush(&mut self) {
        self.seat.client.hush();
    }

    fn close_browser(&mut self) -> Result<(), Failure> {
        self.seat.processes.end_matching(&self.seat.profile_flag)
    }
}

fn health_of(body: &Value) -> SeatHealth {
    let failure = body
        .get("failure")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty());
    SeatHealth {
        failure: failure.map(str::to_owned),
        attached: body.get("attached").and_then(Value::as_bool) == Some(true),
    }
}

pub(super) fn seat_health(_room: &MacRoom) -> Option<SeatHealth> {
    HttpProbe { port: PORT }.health().as_ref().map(health_of)
}

pub(super) fn no_page_message(config: &SeatConfig) -> String {
    format!(
        "The seat page did not attach; check that {} may use the microphone",
        config.browser_name()
    )
}
