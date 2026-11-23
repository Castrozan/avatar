mod control_socket;
mod harness;
mod property;
mod reply_link;
mod voice_activity;

use crate::support::{Outcome, count, replay_cases, rows, text};
use harness::{Harness, frame_of, kind_named};
use serde_json::Value;

type StopPoints = Vec<Vec<usize>>;

fn hear_pair(harness: &mut Harness, pair: &Value) -> Result<bool, Box<dyn std::error::Error>> {
    harness.cells.speaking.set(count(&pair[1])? == 1);
    Ok(harness.hear(&frame_of(kind_named(text(&pair[0])?)?)))
}

fn play_segment(
    harness: &mut Harness,
    segment: &Value,
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    harness.barge.begin_segment();
    let mut stops = Vec::new();
    for (frame_index, pair) in rows(segment)?.iter().enumerate() {
        stops.extend(hear_pair(harness, pair)?.then_some(frame_index));
    }
    Ok(stops)
}

fn play_case(
    case: &Value,
    harness: &mut Harness,
) -> Result<StopPoints, Box<dyn std::error::Error>> {
    let mut stop_points = Vec::new();
    for (segment_index, segment) in rows(&case["segments"])?.iter().enumerate() {
        let frames = play_segment(harness, segment)?;
        stop_points.extend(
            frames
                .into_iter()
                .map(|frame_index| vec![segment_index, frame_index]),
        );
    }
    Ok(stop_points)
}

fn expected_stops(case: &Value) -> Result<StopPoints, Box<dyn std::error::Error>> {
    rows(&case["stops"])?
        .iter()
        .map(|point| rows(point)?.iter().map(count).collect())
        .collect()
}

fn check_stops(case: &Value, stop_points: &StopPoints) -> Outcome {
    let name = text(&case["name"])?;
    assert_eq!(stop_points, &expected_stops(case)?, "{name} stops");
    Ok(())
}

fn check_count(case: &Value, key: &str, actual: usize) -> Outcome {
    let name = text(&case["name"])?;
    assert_eq!(actual, count(&case[key])?, "{name} {key}");
    Ok(())
}

fn check_counters(case: &Value, harness: &Harness) -> Outcome {
    check_count(case, "judged", harness.cells.judged.get())?;
    check_count(case, "resets", harness.cells.resets.get())
}

fn replay_decision(case: &Value) -> Outcome {
    let mut harness = Harness::start();
    let stop_points = play_case(case, &mut harness)?;
    check_stops(case, &stop_points)?;
    check_counters(case, &harness)
}

#[test]
fn every_oracle_decision_case_replays_the_python_barge_in() -> Outcome {
    replay_cases(
        "barge_in/fixtures/decision.json",
        "cases",
        12,
        replay_decision,
    )
}
