use std::collections::VecDeque;

use crate::support::{Outcome, count, flag, replay_cases, rows, text};
use avatar_listening::turn_detector::{TurnFinishedJudge, TurnTracker};
use serde_json::Value;

type Runs = Vec<(u8, usize)>;

fn runs_of(audio: &[u8]) -> Runs {
    audio.iter().fold(Vec::new(), |mut runs: Runs, &byte| {
        match runs.last_mut() {
            Some((value, length)) if *value == byte => *length += 1,
            _ => runs.push((byte, 1)),
        }
        runs
    })
}

fn segment_of(spec: &Value) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let length = count(&spec[0])?;
    let fill = u8::try_from(spec[1].as_u64().ok_or("fill")?)?;
    Ok(vec![fill; length])
}

fn judged_runs(runs: &Value) -> Result<Runs, Box<dyn std::error::Error>> {
    rows(runs)?
        .iter()
        .map(|run| {
            Ok((
                u8::try_from(run[0].as_u64().ok_or("value")?)?,
                count(&run[1])?,
            ))
        })
        .collect()
}

fn flags_of(values: &Value) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
    rows(values)?.iter().map(flag).collect()
}

fn add_segments<Judge: TurnFinishedJudge>(
    tracker: &mut TurnTracker<Judge>,
    case: &Value,
) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
    rows(&case["segments"])?
        .iter()
        .map(|spec| Ok(tracker.add(&segment_of(spec)?)))
        .collect()
}

fn check_tracker(case: &Value, results: &[bool], remaining: usize, judged: &[Runs]) -> Outcome {
    let name = text(&case["name"])?;
    assert_eq!(results, flags_of(&case["results"])?, "{name} results");
    assert_eq!(remaining, count(&case["remaining"])?, "{name} remaining");
    let expected: Vec<Runs> = rows(&case["judged"])?
        .iter()
        .map(judged_runs)
        .collect::<Result<_, _>>()?;
    assert_eq!(judged, expected, "{name} judged");
    Ok(())
}

fn replay_tracker(case: &Value) -> Outcome {
    let mut script: VecDeque<bool> = flags_of(&case["script"])?.into();
    let mut judged = Vec::new();
    let mut tracker = TurnTracker::new(|audio: &[u8]| {
        judged.push(runs_of(audio));
        script.pop_front().unwrap_or(false)
    });
    let results = add_segments(&mut tracker, case)?;
    check_tracker(case, &results, tracker.pending_audio().len(), &judged)
}

#[test]
fn every_oracle_tracker_case_replays_the_python_turn_tracker() -> Outcome {
    replay_cases(
        "turn_detector/fixtures/turn_tracker.json",
        "tracker",
        7,
        replay_tracker,
    )
}
