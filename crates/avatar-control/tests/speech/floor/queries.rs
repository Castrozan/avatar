use avatar_control::speech::floor::{EnqueuedTurn, FloorEvent, SpeechFloor};

use crate::names::TestResult;

#[test]
fn a_turn_holds_the_floor_until_it_is_interrupted() -> TestResult {
    let mut floor = SpeechFloor::default();
    let enqueued = EnqueuedTurn {
        turn_id: 1,
        uses_agent_channel: false,
    };
    floor.handle(FloorEvent::Enqueue(enqueued))?;
    assert!(floor.holds_floor(1));
    floor.handle(FloorEvent::Interrupt)?;
    assert!(!floor.holds_floor(1));
    Ok(())
}
