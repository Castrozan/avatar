use std::fs;
use std::io;
use std::path::Path;

use avatar_conversation::talk::state::seat::parse_seat;
use avatar_floor::quiet::Seat;
use avatar_listening::listener::session::CONFIGURATION_FILE;
use serde_json::Value;

fn invalid(shown: &str) -> io::Error {
    io::Error::other(format!("{shown} is not a valid Seat"))
}

fn configuration_of(directory: &Path) -> io::Result<Value> {
    match fs::read_to_string(directory.join(CONFIGURATION_FILE)) {
        Ok(text) => serde_json::from_str(&text).map_err(io::Error::other),
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => Ok(Value::Null),
        Err(failure) => Err(failure),
    }
}

pub(super) fn configured_seat(directory: &Path) -> io::Result<Seat> {
    match configuration_of(directory)?.get("seat") {
        None => Ok(Seat::Room),
        Some(Value::String(name)) => parse_seat(name).ok_or_else(|| invalid(name)),
        Some(other) => Err(invalid(&other.to_string())),
    }
}
