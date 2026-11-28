use avatar_floor::addressee::Addressee;
use avatar_live::boundaries::HolderLookup;
use avatar_live::voices::voice_of_the_room_holder;
use serde_json::{Value, json};

use crate::support::{Outcome, VirtualClock, addressee, block_on, number, same, try_cases};

const FIXTURE: &str = include_str!("fixtures/holder_wait.json");

struct AppearsLater {
    after_polls: Option<u64>,
    lookups: u64,
}

fn anyone() -> Addressee {
    addressee("w:p", "anyone")
}

impl HolderLookup for AppearsLater {
    fn current(&mut self) -> Option<Addressee> {
        self.lookups += 1;
        let appeared = self.after_polls.is_some_and(|polls| self.lookups > polls);
        appeared.then(anyone)
    }
}

fn any_voice(_: &str) -> String {
    "EXAVITQu4vr4xnSDxMaL".to_owned()
}

async fn wait(case: &Value) -> (String, u64, f64) {
    let after_polls = case["appears_after_polls"].as_u64();
    let mut holders = AppearsLater {
        after_polls,
        lookups: 0,
    };
    let mut clock = VirtualClock::default();
    let voice = voice_of_the_room_holder(&mut holders, &mut clock, any_voice).await;
    (voice, holders.lookups, clock.now)
}

fn check(case: &Value) -> Outcome {
    let (voice, lookups, now) = block_on(wait(case))?;
    same(
        &json!([voice, lookups]),
        &json!([case["voice"], case["lookups"]]),
        case,
    );
    assert_eq!(now.to_bits(), number(&case["clock"]).to_bits());
    Ok(())
}

#[test]
fn every_recorded_python_holder_wait_is_reproduced() -> Outcome {
    try_cases(FIXTURE, 7, check)
}
