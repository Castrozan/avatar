use avatar_seat::edges::{HttpProbe, SeatProbe as HealthProbe, SystemClock, SystemProcesses};
use avatar_seat::local::{LocalSeat, SEAT_PORT};
use serde_json::Value;

use super::MacRoom;
use crate::edge::failure::edge_failure;
use crate::failure::Failure;
use crate::talk::SeatProbe;

fn flag(key: &str) -> bool {
    let health = HttpProbe { port: SEAT_PORT }.health();
    health.and_then(|body| body.get(key).and_then(Value::as_bool)) == Some(true)
}

fn local_seat_of<Output>(room: &MacRoom, body: impl FnOnce(&LocalSeat<'_>) -> Output) -> Output {
    let clock = SystemClock::new();
    let probe = HttpProbe { port: SEAT_PORT };
    body(&LocalSeat::new(
        &room.config,
        &SystemProcesses,
        &clock,
        &probe,
    ))
}

fn kept_here(seat: &LocalSeat<'_>) -> Result<bool, Failure> {
    seat.meeting_seat_kept_here()
        .map_err(|error| edge_failure(&error))
}

fn leave(seat: &LocalSeat<'_>) -> Result<(), Failure> {
    seat.leave().map_err(|error| edge_failure(&error))
}

impl SeatProbe for MacRoom {
    fn seat_reachable(&mut self) -> bool {
        HttpProbe { port: SEAT_PORT }.health().is_some()
    }

    fn conversation_listening(&mut self) -> bool {
        flag("listening")
    }

    fn meeting_held(&mut self) -> bool {
        flag("meeting")
    }

    fn meeting_seat_kept_here(&mut self) -> Result<bool, Failure> {
        local_seat_of(self, kept_here)
    }

    fn leave_seat(&mut self) -> Result<(), Failure> {
        local_seat_of(self, leave)
    }

    fn seat_did_not_attach_message(&self) -> String {
        let browser = self.config.browser_name();
        format!("The conversation page did not attach; check that {browser} may use the microphone")
    }
}
