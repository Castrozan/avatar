use std::env::current_exe;
use std::path::PathBuf;

use avatar_mac::client::{CALL_DEVICE, MEETING_FLAG, MEETING_MICROPHONE};

const PROGRAM: &str = "avatar";

fn program() -> String {
    let path = current_exe().unwrap_or_else(|_| PathBuf::from(PROGRAM));
    path.to_string_lossy().into_owned()
}

fn seat_words(device: &str) -> Vec<&'static str> {
    match device {
        CALL_DEVICE => vec!["mac", "call-capture"],
        MEETING_MICROPHONE => vec!["mac", "seat", MEETING_FLAG],
        _ => vec!["mac", "seat"],
    }
}

#[must_use]
pub(crate) fn mac_capture_command(device: &str) -> Vec<String> {
    let words = seat_words(device).into_iter().map(str::to_owned);
    std::iter::once(program()).chain(words).collect()
}
