use std::cell::{Cell, RefCell};
use std::error::Error;

use avatar_wake::peer::listing::{PeerListing, sessions_of_peer};
use avatar_wake::reach::CommandRunner;
use serde_json::{Value, json};

use super::support::{cases, optional_text, session_json, text};

const FIXTURE: &str = include_str!("fixtures/peer.json");
const PINNED_PEER_TIMEOUT_SECONDS: u64 = 10;

#[derive(Default)]
struct ReplyingRunner {
    reply: Option<String>,
    command: RefCell<Vec<String>>,
    timeout_seconds: Cell<u64>,
}

impl CommandRunner for ReplyingRunner {
    fn stdout_of(&self, command: &[String], timeout_seconds: u64) -> Option<String> {
        self.command.replace(command.to_vec());
        self.timeout_seconds.set(timeout_seconds);
        self.reply.clone()
    }
}

fn listing_json(listing: Option<&PeerListing>) -> Value {
    listing.map_or(Value::Null, |listed| {
        json!({
            "sessions": listed.sessions.iter().map(session_json).collect::<Vec<_>>(),
            "conversation_seat": listed.conversation_seat,
        })
    })
}

#[test]
fn a_peer_listing_is_parsed_or_contributes_nothing() -> Result<(), Box<dyn Error>> {
    cases(FIXTURE, "listing")?
        .iter()
        .for_each(check_listing_case);
    Ok(())
}

fn check_listing_case(case: &Value) {
    let runner = ReplyingRunner {
        reply: optional_text(&case["reply"]),
        ..ReplyingRunner::default()
    };
    let listing = sessions_of_peer(&text(&case["host"]), &runner);
    assert_eq!(listing_json(listing.as_ref()), case["expected"]);
    assert_eq!(runner.timeout_seconds.get(), PINNED_PEER_TIMEOUT_SECONDS);
}

#[test]
fn the_peer_is_asked_over_ssh_with_a_bounded_wait() {
    let runner = ReplyingRunner::default();
    assert!(sessions_of_peer("kira", &runner).is_none());
    assert_eq!(
        runner.command.borrow()[..5],
        ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=5"]
    );
    assert_eq!(
        runner.command.borrow()[5..],
        ["kira", "avatar-wake", "--sessions"]
    );
}

#[test]
fn the_discord_seat_is_the_only_seat_that_names_a_discord_call() {
    let listing = PeerListing {
        sessions: Vec::new(),
        conversation_seat: Some("discord".to_owned()),
    };
    assert!(listing.is_on_discord());
}
