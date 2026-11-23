use std::thread;
use std::time::Duration;

use avatar_adapters::clock::monotonic_seconds;
use avatar_adapters::commands::{Programs, TalkProcess};
use avatar_conversation::talk::surfaces::discord::listener_left_the_call;
use avatar_floor::quiet::Seat;
use avatar_routing::names::session::Session;
use avatar_wake::addressing::{DecisionUnavailable, answers};
use avatar_wake::arbitration::conversation_hearing::room_that_heard;
use avatar_wake::listener::{Boundary, BoundaryFailure, WakeEffects};
use serde_json::Value;

use super::Machine;
use super::claims::claim_here;
use super::commands::{Shell, exit_status, on_host, unavailable};
use super::sessions::local_sessions;
use super::transcriber::transcribed;
use crate::support::wake_log::{report_to_log, room_name, utc_timestamp};

const COMMAND_SECONDS: u64 = 120;
const PEER_CLAIM_SECONDS: u64 = 20;
const PEER_HEAD_START: Duration = Duration::from_secs(1);

fn agent_command(host: Option<&str>, words: &[&str]) -> Boundary<i32> {
    exit_status(&on_host(host, words), COMMAND_SECONDS).map_err(|error| unavailable(&error))
}

fn claimed_at_servant_machine(session: &Session, room: &str) -> bool {
    thread::sleep(PEER_HEAD_START);
    let words = ["avatar-wake", "--claim", &session.pane, "--room", room];
    let command = on_host(session.host.as_deref(), &words);
    exit_status(&command, PEER_CLAIM_SECONDS).is_ok_and(|code| code == 0)
}

impl WakeEffects for Machine {
    fn sessions(&self) -> Vec<Session> {
        let local = local_sessions();
        avatar_wake::peer::listing::sessions_everywhere(&local, &self.peers.sessions())
    }

    fn discord_call_host(&self) -> Option<String> {
        self.peers.discord_call_host()
    }

    fn open_seat(&self) -> Option<Seat> {
        self.open_conversation_seat()
    }

    fn discord_call_ended(&self) -> bool {
        listener_left_the_call(&mut TalkProcess::new(Programs::on_path()))
    }

    fn transcribe(&self, segment: &[u8]) -> Boundary<String> {
        transcribed(&self.runtime, segment).map_err(BoundaryFailure::Unavailable)
    }

    fn decide(&self, input: &Value, questions: &Value) -> Result<Value, DecisionUnavailable> {
        answers(&self.decisions, input, questions)
    }

    fn room_that_heard(&self, seconds: f64) -> Option<String> {
        room_that_heard(&self.peers.answering_hosts(), seconds, &Shell)
    }

    fn claim_wake(&self, session: &Session) -> bool {
        let room = room_name().unwrap_or_default();
        match session.host {
            Some(_) => claimed_at_servant_machine(session, &room),
            None => claim_here(&self.directory, &room, &session.pane, &room).unwrap_or(false),
        }
    }

    fn prompt_now(&self, pane: &str, text: &str, host: Option<&str>) -> Boundary<bool> {
        let words = ["herdr", "agent", "prompt", pane, text];
        agent_command(host, &words).map(|code| code == 0)
    }

    fn notify_at_idle(&self, pane: &str, text: &str, host: Option<&str>) -> Boundary<()> {
        let words = ["a2a", "notify", "--sender", "voice", pane, text];
        agent_command(host, &words).map(|_| ())
    }

    fn close_conversation(&self) {
        drop(agent_command(None, &["avatar-talk", "close"]));
    }

    fn report(&self, line: &str) {
        report_to_log(line);
    }

    fn room_name(&self) -> Option<String> {
        room_name()
    }

    fn timestamp(&self) -> String {
        utc_timestamp()
    }

    fn now(&self) -> f64 {
        monotonic_seconds()
    }

    fn monotonic(&self) -> f64 {
        monotonic_seconds()
    }

    fn sleep(&self, seconds: f64) {
        self.pause(seconds);
    }

    fn pause(&self, seconds: f64) {
        thread::sleep(Duration::try_from_secs_f64(seconds).unwrap_or_default());
    }
}
