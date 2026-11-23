use std::io::{self, Write};
use std::process::ExitCode;

use avatar_adapters::clock::wall_seconds;
use avatar_floor::claim::conversation_report;
use avatar_wake::arbitration::conversation_hearing::{ListeningSnapshot, heard_within};
use avatar_wake::listener::{ConversationSeat, WakeEffects};
use avatar_wake::peer::listing::listing_of;
use serde_json::Value;

use super::machine::{Machine, claim_here, heard_entries, local_sessions, with_site};

const USAGE_ERROR: u8 = 2;

fn print(text: &str) -> io::Result<ExitCode> {
    writeln!(io::stdout().lock(), "{text}")?;
    Ok(ExitCode::SUCCESS)
}

fn refuse(message: &str, code: u8) -> io::Result<ExitCode> {
    writeln!(io::stderr().lock(), "{message}")?;
    Ok(ExitCode::from(code))
}

pub(super) fn conversation(machine: &Machine) -> io::Result<ExitCode> {
    let host = machine.room_name().unwrap_or_default();
    let report = with_site(
        machine.capture(),
        machine.directory(),
        &host,
        conversation_report,
    )?;
    print(&report)
}

pub(super) fn sessions(machine: &Machine) -> io::Result<ExitCode> {
    print(&listing_of(&local_sessions(), machine.open_seat()))
}

pub(super) fn claim(machine: &Machine, pane: &str, room: Option<&str>) -> io::Result<ExitCode> {
    let Some(room) = room.filter(|name| !name.is_empty()) else {
        return refuse("--claim needs --room", USAGE_ERROR);
    };
    let host = machine.room_name().unwrap_or_default();
    match claim_here(machine.capture(), machine.directory(), (&host, pane, room)) {
        Ok(won) => Ok(ExitCode::from(u8::from(!won))),
        Err(error) => refuse(&format!("Wake claim refused: {error}"), USAGE_ERROR),
    }
}

pub(super) fn heard(machine: &Machine, seconds: f64) -> io::Result<ExitCode> {
    let directory = machine.directory();
    let snapshot = ListeningSnapshot {
        listening: machine.conversation_listening(),
        speech_in_progress: machine.speech_in_progress(),
        entries: heard_entries(directory),
    };
    print(&Value::Bool(heard_within(&snapshot, seconds, wall_seconds())).to_string())
}
