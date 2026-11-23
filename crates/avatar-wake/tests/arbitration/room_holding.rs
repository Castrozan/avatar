use std::cell::{Cell, RefCell};
use std::error::Error;

use avatar_routing::names::session::Session;
use avatar_wake::arbitration::room_holding::{HeardSegment, HoldingBoundaries, holds_the_room};
use serde_json::{Value, json};

use super::support::{cases, number, optional_text};

const FIXTURE: &str = include_str!("fixtures/arbitration_room.json");

#[derive(Default)]
struct HoldingFake {
    discord_host: Option<String>,
    room_host: Option<String>,
    now: f64,
    claim: bool,
    seconds_asked: Cell<Option<f64>>,
    claim_asked: Cell<bool>,
    reports: RefCell<Vec<String>>,
}

impl HoldingBoundaries for HoldingFake {
    fn discord_call_host(&self) -> Option<String> {
        self.discord_host.clone()
    }

    fn room_that_heard(&self, seconds: f64) -> Option<String> {
        self.seconds_asked.set(Some(seconds));
        self.room_host.clone()
    }

    fn now(&self) -> f64 {
        self.now
    }

    fn claim_wake(&self, _session: &Session) -> bool {
        self.claim_asked.set(true);
        self.claim
    }

    fn report(&self, text: &str) {
        self.reports.borrow_mut().push(text.to_owned());
    }
}

#[test]
fn holding_the_room_follows_the_python_arbitration() -> Result<(), Box<dyn Error>> {
    let holding = cases(FIXTURE, "holding")?;
    assert!(holding.len() >= 40, "the oracle covers at least 40 cases");
    holding.iter().for_each(check_holding_case);
    Ok(())
}

fn check_holding_case(case: &Value) {
    let fake = fake_of(case);
    let heard = heard_of(case);
    let holds = holds_the_room(&fake, case["dry_run"] == json!(true), &session(), &heard);
    let expected = &case["expected"];
    assert_eq!(json!(holds), expected["holds"]);
    assert_eq!(json!(*fake.reports.borrow()), expected["reports"]);
    assert_eq!(json!(fake.claim_asked.get()), expected["claim_asked"]);
    let asked = fake.seconds_asked.get().map(f64::to_bits);
    assert_eq!(asked, expected["seconds_asked"].as_f64().map(f64::to_bits));
}

fn fake_of(case: &Value) -> HoldingFake {
    HoldingFake {
        discord_host: optional_text(&case["discord_host"]),
        room_host: optional_text(&case["room_host"]),
        now: number(&case["now"]),
        claim: case["claim"] == json!(true),
        ..HoldingFake::default()
    }
}

fn heard_of(case: &Value) -> HeardSegment {
    let byte_count = case["audio_byte_count"].as_u64().unwrap_or_default();
    HeardSegment {
        audio_byte_count: u32::try_from(byte_count).unwrap_or_default(),
        heard_at: number(&case["heard_at"]),
    }
}

fn session() -> Session {
    Session {
        servant: "Medea".to_owned(),
        pane: "w1:p2".to_owned(),
        host: Some("kira".to_owned()),
    }
}
