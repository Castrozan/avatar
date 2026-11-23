use serde::Deserialize;
use serde_json::{Value, json};

use crate::quiet::{ConversationTranscript, Entry};

#[derive(Deserialize)]
struct Script {
    script: Vec<Step>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Step {
    NewEntries(Vec<Entry>),
    Start(Vec<Entry>),
    CompletedTurn(CompletedTurn),
    TurnEndedInQuiet(QuietCheck),
}

#[derive(Deserialize)]
struct CompletedTurn {
    entry: Entry,
    heard_at: f64,
}

#[derive(Deserialize)]
struct QuietCheck {
    now: f64,
    speech_in_progress: bool,
}

pub(super) fn run(request: &Value) -> Result<Value, String> {
    let script = Script::deserialize(request).map_err(|error| error.to_string())?;
    let mut transcript = ConversationTranscript::default();
    let outputs: Vec<Value> = script
        .script
        .into_iter()
        .map(|step| replay(&mut transcript, step))
        .collect();
    Ok(Value::Array(outputs))
}

fn replay(transcript: &mut ConversationTranscript, step: Step) -> Value {
    match step {
        Step::NewEntries(entries) => json!(transcript.new_entries(&entries)),
        Step::Start(entries) => start(transcript, &entries),
        Step::CompletedTurn(turn) => json!(transcript.completed_turn(&turn.entry, turn.heard_at)),
        Step::TurnEndedInQuiet(check) => json!(quiet_turn(transcript, &check)),
    }
}

fn start(transcript: &mut ConversationTranscript, entries: &[Entry]) -> Value {
    transcript.start(entries);
    Value::Null
}

fn quiet_turn(transcript: &mut ConversationTranscript, check: &QuietCheck) -> Option<Entry> {
    transcript.turn_ended_in_quiet(check.now, || check.speech_in_progress)
}
