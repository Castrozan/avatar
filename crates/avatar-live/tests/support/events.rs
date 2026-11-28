use avatar_live::events::{Audio, LiveEvent, TaskRequest};
use serde_json::{Map, Value, json};

use super::{bytes, text};

type Fields = Vec<(&'static str, Value)>;

fn tagged(kind: &str, fields: Fields) -> Value {
    let mut object = Map::new();
    object.insert("kind".to_owned(), json!(kind));
    object.extend(
        fields
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value)),
    );
    Value::Object(object)
}

fn task_fields(task: &TaskRequest) -> Fields {
    vec![
        ("id", json!(task.task_id)),
        ("request", json!(task.request)),
    ]
}

fn audio_fields(audio: &Audio) -> Fields {
    vec![("pcm", json!(audio.pcm)), ("voiced", json!(audio.voiced()))]
}

pub(crate) fn event_json(event: &LiveEvent) -> Value {
    match event {
        LiveEvent::Audio(audio) => tagged("audio", audio_fields(audio)),
        LiveEvent::UserSaid(words) => tagged("user_said", vec![("text", json!(words))]),
        LiveEvent::AssistantSaid(words) => tagged("assistant_said", vec![("text", json!(words))]),
        LiveEvent::TaskRequested(task) => tagged("task", task_fields(task)),
        LiveEvent::Interrupted => tagged("interrupted", vec![]),
        LiveEvent::Failed(message) => tagged("failed", vec![("message", json!(message))]),
    }
}

fn task_of(spec: &Value) -> TaskRequest {
    TaskRequest {
        task_id: text(&spec["id"]).to_owned(),
        request: text(&spec["request"]).to_owned(),
    }
}

fn audio_of(spec: &Value) -> Audio {
    Audio {
        pcm: bytes(&spec["pcm"]),
    }
}

pub(crate) fn event_from(spec: &Value) -> LiveEvent {
    let words = text(&spec["text"]).to_owned();
    match text(&spec["kind"]) {
        "audio" => LiveEvent::Audio(audio_of(spec)),
        "user_said" => LiveEvent::UserSaid(words),
        "assistant_said" => LiveEvent::AssistantSaid(words),
        "task" => LiveEvent::TaskRequested(task_of(spec)),
        "interrupted" => LiveEvent::Interrupted,
        _ => LiveEvent::Failed(text(&spec["message"]).to_owned()),
    }
}
