use avatar_live::events::{Audio, LiveEvent, TaskRequest};
use avatar_live::ordered_json;
use serde_json::{Map, Value, json};

use super::{bytes, plain, text};

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

pub(crate) fn event_json(event: &LiveEvent) -> Result<Value, serde_json::Error> {
    Ok(match event {
        LiveEvent::Audio(audio) => tagged("audio", audio_fields(audio)),
        LiveEvent::UserSaid(words) => tagged("user_said", vec![("text", json!(words))]),
        LiveEvent::AssistantSaid(words) => tagged("assistant_said", vec![("text", json!(words))]),
        LiveEvent::TaskRequested(task) => tagged("task", task_fields(task)),
        LiveEvent::Usage(details) => tagged("usage", vec![("details", plain(details)?)]),
        LiveEvent::Failed(message) => tagged("failed", vec![("message", json!(message))]),
    })
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

pub(crate) fn event_from(spec: &Value) -> Result<LiveEvent, Box<dyn std::error::Error>> {
    let words = text(&spec["text"]).to_owned();
    Ok(match text(&spec["kind"]) {
        "audio" => LiveEvent::Audio(audio_of(spec)),
        "user_said" => LiveEvent::UserSaid(words),
        "assistant_said" => LiveEvent::AssistantSaid(words),
        "task" => LiveEvent::TaskRequested(task_of(spec)),
        "usage" => LiveEvent::Usage(ordered_json::parse(&spec["details"].to_string())?),
        _ => LiveEvent::Failed(text(&spec["message"]).to_owned()),
    })
}
