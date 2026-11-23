use avatar_floor::quiet::Seat;
use serde_json::Value;

use super::configuration::read_configuration;
use super::phase::live_purpose;
use crate::talk::failure::{TalkFailure, python_text_repr};
use crate::talk::ports::{Files, Units};

#[must_use]
pub const fn seat_name(seat: Seat) -> &'static str {
    match seat {
        Seat::Room => "room",
        Seat::Remote => "remote",
        Seat::Discord => "discord",
    }
}

#[must_use]
pub fn parse_seat(text: &str) -> Option<Seat> {
    [Seat::Room, Seat::Remote, Seat::Discord]
        .into_iter()
        .find(|seat| seat_name(*seat) == text)
}

#[must_use]
pub const fn microphone_source(seat: Seat) -> &'static str {
    match seat {
        Seat::Room => "AvatarRoomMicrophone",
        Seat::Remote => "AvatarRemoteMicrophone.monitor",
        Seat::Discord => "discord",
    }
}

#[must_use]
pub const fn speech_output(seat: Seat) -> &'static str {
    match seat {
        Seat::Room => "speakers",
        Seat::Remote => "remote",
        Seat::Discord => "discord",
    }
}

#[must_use]
pub fn seat_microphone_ready(seat: Seat, existing_sources: &[String]) -> bool {
    seat == Seat::Discord
        || existing_sources
            .iter()
            .any(|source| source == microphone_source(seat))
}

pub fn configured(files: &mut impl Files) -> Result<Seat, TalkFailure> {
    match read_configuration(files)?.get("seat") {
        None => Ok(Seat::Room),
        Some(Value::String(name)) => {
            parse_seat(name).ok_or_else(|| invalid_seat(&python_text_repr(name)))
        }
        Some(other) => Err(invalid_seat(&other.to_string())),
    }
}

fn invalid_seat(shown: &str) -> TalkFailure {
    TalkFailure::new(format!("{shown} is not a valid Seat"))
}

pub fn current(boundary: &mut (impl Units + Files)) -> Result<Seat, TalkFailure> {
    if live_purpose(boundary).is_none() {
        return Ok(Seat::Room);
    }
    configured(boundary)
}
