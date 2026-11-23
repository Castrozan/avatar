use std::cell::Cell;
use std::path::Path;

use super::doubles::{Identities, Script, Shared, Ticks};
use super::inputs::{Failure, Inputs};
use crate::edges::session::daemon::fakes::{Events, Notes, Queue, StopAfter};
use crate::support::{Outcome, rows, text};
use avatar_listening::listener::daemon::{RecorderParts, TranscriptRecorder, write_transcript};
use serde_json::Value;

type Recorder = TranscriptRecorder<Script, Identities, Ticks>;

#[derive(Default)]
struct Observed {
    calls: Shared<(usize, Option<String>)>,
    pauses: Shared<f64>,
}

fn assert_file(path: &Path, expected: &Value) {
    let written = std::fs::read_to_string(path).ok();
    assert_eq!(written.as_deref(), expected.as_str(), "{}", path.display());
}

fn strings(value: &Value) -> Result<Vec<String>, Failure> {
    rows(value)?
        .iter()
        .map(|item| Ok(text(item)?.to_owned()))
        .collect()
}

fn floats(value: &Value) -> Result<Vec<f64>, Failure> {
    let numbers = value.as_array().ok_or("floats")?.iter();
    Ok(numbers.filter_map(Value::as_f64).collect())
}

fn recorder_of(scenario: &Value, inputs: &mut Inputs, root: &Path, seen: &Observed) -> Recorder {
    let outcomes = std::mem::take(&mut inputs.outcomes);
    TranscriptRecorder::new(RecorderParts {
        transcript_path: root.join("transcript.jsonl"),
        embeddings_path: root.join("embeddings.jsonl"),
        language: scenario["language"].as_str().map(str::to_owned),
        transcriber: Script::new(outcomes, &seen.calls),
        identifier: Identities(std::mem::take(&mut inputs.identities)),
        timing: Ticks::new(&seen.pauses),
    })
}

fn drive(recorder: &mut Recorder, inputs: Inputs) -> (Events, Notes) {
    let (events, mut notes) = (Events::default(), Notes::default());
    let stopping = StopAfter(Cell::new(inputs.runs));
    write_transcript(
        recorder,
        &mut Queue(inputs.items),
        &stopping,
        &events,
        &mut notes,
    );
    (events, notes)
}

fn assert_outputs(root: &Path, scenario: &Value, run: (&Events, &Notes)) -> Outcome {
    assert_file(&root.join("transcript.jsonl"), &scenario["transcript"]);
    assert_file(&root.join("embeddings.jsonl"), &scenario["embeddings"]);
    assert_eq!(run.0.recorded(), strings(&scenario["events"])?);
    assert_eq!(run.1.0, strings(&scenario["stderr"])?);
    Ok(())
}

fn seed_transcript(root: &Path, scenario: &Value) -> Outcome {
    let seed = scenario["seed"].as_str();
    seed.map(|text| std::fs::write(root.join("transcript.jsonl"), text))
        .transpose()?;
    Ok(())
}

fn assert_observed(seen: &Observed, scenario: &Value) -> Outcome {
    assert_eq!(*seen.pauses.borrow(), floats(&scenario["sleeps"])?);
    assert_eq!(seen.calls.borrow().len(), rows(&scenario["calls"])?.len());
    Ok(())
}

pub(super) fn replay(scenario: &Value) -> Outcome {
    let root = tempfile::tempdir()?;
    seed_transcript(root.path(), scenario)?;
    let (mut inputs, seen) = (Inputs::of(scenario)?, Observed::default());
    let mut recorder = recorder_of(scenario, &mut inputs, root.path(), &seen);
    let (events, notes) = drive(&mut recorder, inputs);
    assert_outputs(root.path(), scenario, (&events, &notes))?;
    assert_observed(&seen, scenario)
}
