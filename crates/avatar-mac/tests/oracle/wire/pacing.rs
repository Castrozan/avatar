use avatar_mac::frame::whole_samples;
use avatar_mac::seat::{PacingStep, SpeechPacing};
use serde_json::{Value, json};

use crate::support::{bits, each, entries, parse};

#[derive(Default)]
struct Progress {
    at: usize,
    sends: u64,
    interrupted: bool,
    events: Vec<Value>,
}

struct Run {
    pacing: SpeechPacing,
    times: Vec<f64>,
    stops: [Option<u64>; 2],
    progress: Progress,
}

fn next_time(run: &mut Run) -> f64 {
    run.progress.at += 1;
    run.times.get(run.progress.at - 1).copied().unwrap_or(0.0)
}

fn after_send(run: &mut Run, size: usize) {
    run.progress.sends += 1;
    run.progress.events.push(json!(["send", size]));
    run.progress.interrupted |= run.stops.contains(&Some(run.progress.sends));
    let now = next_time(run);
    run.progress
        .events
        .push(json!(["sleep", run.pacing.pause_after_send(now)]));
}

fn advance(run: &mut Run) -> bool {
    match run.pacing.next(run.progress.interrupted) {
        PacingStep::Send { start, end } => after_send(run, end - start),
        PacingStep::Cushion(seconds) => run.progress.events.push(json!(["sleep", seconds])),
        PacingStep::Finished => return false,
    }
    true
}

fn times_of(case: &Value) -> Vec<f64> {
    entries(&case["times"])
        .iter()
        .filter_map(Value::as_f64)
        .collect()
}

fn pacing_of(case: &Value, times: &[f64]) -> SpeechPacing {
    let length = usize::try_from(case["length"].as_u64().unwrap_or(0)).unwrap_or(0);
    SpeechPacing::new(whole_samples(&vec![0; length]).len(), times[0])
}

fn stops_of(case: &Value) -> [Option<u64>; 2] {
    [case["hush_after"].as_u64(), case["leave_after"].as_u64()]
}

fn start(case: &Value) -> Run {
    let times = times_of(case);
    let pacing = pacing_of(case, &times);
    let mut run = Run {
        pacing,
        times,
        stops: stops_of(case),
        progress: Progress::default(),
    };
    next_time(&mut run);
    run
}

fn comparable(entry: &Value) -> (Value, Option<u64>) {
    (entry[0].clone(), bits(&entry[1]))
}

fn check_pacing(case: &Value) {
    let mut run = start(case);
    while advance(&mut run) {}
    let ours: Vec<_> = run.progress.events.iter().map(comparable).collect();
    let theirs: Vec<_> = entries(&case["events"]).iter().map(comparable).collect();
    assert_eq!(ours, theirs, "{case}");
}

#[test]
fn speech_is_paced_as_the_python_seat_paces_it() {
    each(
        &parse(include_str!("../fixtures/seat.json"))["pacing"],
        check_pacing,
    );
}
