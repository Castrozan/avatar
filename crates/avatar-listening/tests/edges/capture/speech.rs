use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::desk::{Desk, Hearing, Log, Queued};
use super::frames::frame_of;
use crate::edges::helpers::Failure;
use crate::support::{Outcome, count, fixture, number, rows, text};
use avatar_listening::listener::capture::{HeardMoments, SourceSpeech};
use serde_json::Value;

thread_local! {
    static READS: Cell<u32> = const { Cell::new(0) };
}

fn scripted_now() -> f64 {
    let read = READS.with(|reads| reads.replace(reads.get() + 1));
    5000.0 + f64::from(read) * 0.5
}

fn heard_of(item: &Value) -> Result<HeardMoments, Failure> {
    Ok(HeardMoments {
        speech_ended: number(&item["speech_ended"])?,
        segment_closed: number(&item["segment_closed"])?,
    })
}

fn queued_of(item: &Value) -> Result<Queued, Failure> {
    Ok(Queued {
        source: text(&item["source"])?.to_owned(),
        length: count(&item["length"])?,
        sum: item["sum"].as_u64().ok_or("sum")?,
        turn_finished: item["turn_finished"] == true,
        heard: heard_of(item)?,
    })
}

fn expected_queue(case: &Value) -> Result<Vec<Queued>, Failure> {
    rows(&case["queued"])?.iter().map(queued_of).collect()
}

type Speech = SourceSpeech<fn(&[u8]) -> bool, Hearing, Desk>;

fn judge(audio: &[u8]) -> bool {
    (audio.len() / 1024).is_multiple_of(3)
}

fn feed_pattern(case: &Value, speech: &mut Speech) -> Outcome {
    let seed = case["seed"].as_u64().ok_or("seed")?;
    for (index, kind) in text(&case["pattern"])?.chars().enumerate() {
        speech.feed(&frame_of(kind, seed + u64::try_from(index)?));
    }
    speech.abandon();
    Ok(())
}

#[derive(Default)]
struct Run {
    log: Log,
    queue: Rc<RefCell<Vec<Queued>>>,
}

fn speech_over(run: &Run, capacity: usize) -> Speech {
    let desk = Desk::new(&run.log, capacity, &run.queue);
    Speech::new(
        "meeting",
        judge,
        Hearing(run.log.clone()),
        desk,
        scripted_now,
    )
}

fn run_case(case: &Value) -> Result<Run, Failure> {
    READS.with(|reads| reads.set(0));
    let run = Run::default();
    feed_pattern(case, &mut speech_over(&run, count(&case["capacity"])?))?;
    Ok(run)
}

fn replay(case: &Value) -> Outcome {
    let run = run_case(case)?;
    assert_eq!(
        Value::Array(run.log.borrow().clone()),
        case["log"],
        "{case}"
    );
    assert_eq!(*run.queue.borrow(), expected_queue(case)?, "{case}");
    Ok(())
}

#[test]
fn every_oracle_source_speech_matches_the_python_listener() -> Outcome {
    let document = fixture("edges/fixtures/capture.json")?;
    let cases = rows(&document["cases"])?;
    assert_eq!(cases.len(), 18);
    cases.iter().try_for_each(replay)
}
