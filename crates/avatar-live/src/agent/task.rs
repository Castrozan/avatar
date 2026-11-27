use serde_json::Value;

use super::text::{spoken_answer, tagged_answer};

const TERMINAL_STATES: [&str; 3] = ["completed", "canceled", "failed"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Answer(String),
    Failure(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AgentTask {
    pub state: Option<String>,
    pub output: String,
    pub error_message: Option<String>,
}

fn string_field(task: &Value, key: &str) -> Option<String> {
    task.get(key).and_then(Value::as_str).map(str::to_owned)
}

#[must_use]
pub fn parse_task(output: &str) -> AgentTask {
    let Ok(task) = serde_json::from_str::<Value>(output) else {
        return AgentTask::default();
    };
    AgentTask {
        state: string_field(&task, "state"),
        output: string_field(&task, "output").unwrap_or_default(),
        error_message: string_field(&task, "errorMessage"),
    }
}

fn failure_reason(task: &AgentTask) -> String {
    match task.error_message.as_deref() {
        Some(message) if !message.is_empty() => message.to_owned(),
        _ => format!("its task ended {}", task.state.as_deref().unwrap_or("None")),
    }
}

fn finished_verdict(task: &AgentTask) -> Verdict {
    let (answer, _) = spoken_answer(&task.output);
    match answer {
        Some(spoken) if task.state.as_deref() == Some("completed") && !spoken.is_empty() => {
            Verdict::Answer(spoken)
        }
        _ => Verdict::Failure(failure_reason(task)),
    }
}

#[must_use]
pub fn verdict_in(task: &AgentTask, tag: &str) -> Option<Verdict> {
    if let Some(tagged) = tagged_answer(&task.output, tag) {
        return Some(Verdict::Answer(tagged));
    }
    let terminal = task
        .state
        .as_deref()
        .is_some_and(|state| TERMINAL_STATES.contains(&state));
    terminal.then(|| finished_verdict(task))
}
