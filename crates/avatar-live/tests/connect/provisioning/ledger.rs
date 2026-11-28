use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

pub(super) const KEY: &str = "test-key";

#[derive(Default)]
pub(super) struct Ledger {
    pub(super) tools: Vec<Value>,
    pub(super) agents: Vec<Value>,
    pub(super) writes: Vec<String>,
}

pub(super) type Answer = (StatusCode, Value);

fn identifier(collection: &str) -> &'static str {
    match collection {
        "tools" => "id",
        _ => "agent_id",
    }
}

fn missing() -> Answer {
    (StatusCode::NOT_FOUND, json!({"detail": "not found"}))
}

fn merged(mut item: Value, changes: &Value) -> Value {
    let fields = changes.as_object().into_iter().flatten();
    fields.for_each(|(key, value)| item[key] = value.clone());
    item
}

impl Ledger {
    fn items(&mut self, collection: &str) -> &mut Vec<Value> {
        match collection {
            "tools" => &mut self.tools,
            _ => &mut self.agents,
        }
    }

    fn list(&mut self, collection: &str) -> Answer {
        let items = self.items(collection).clone();
        (
            StatusCode::OK,
            json!({collection: items, "has_more": false}),
        )
    }

    fn create(&mut self, collection: &str, body: &Value) -> Answer {
        let key = identifier(collection);
        let id = format!("{collection}-{}", self.items(collection).len() + 1);
        self.items(collection).push(merged(json!({key: id}), body));
        self.writes.push(format!("create {collection}"));
        (StatusCode::OK, json!({key: id}))
    }

    fn position(&mut self, collection: &str, id: &str) -> Option<usize> {
        let key = identifier(collection);
        self.items(collection)
            .iter()
            .position(|item| item[key] == id)
    }

    fn item(&mut self, collection: &str, id: &str, changes: Option<&Value>) -> Answer {
        let index = self.position(collection, id);
        index.map_or_else(missing, |found| self.update(collection, found, changes))
    }

    fn update(&mut self, collection: &str, index: usize, changes: Option<&Value>) -> Answer {
        let current = self.items(collection)[index].clone();
        let updated = changes.map_or_else(|| current.clone(), |body| merged(current.clone(), body));
        self.items(collection)[index] = updated.clone();
        self.writes
            .extend(changes.map(|_| format!("patch {collection}")));
        (StatusCode::OK, updated)
    }

    pub(super) fn answer(&mut self, method: &Method, path: &str, body: &Value) -> Answer {
        let segments: Vec<&str> = path.trim_start_matches("/v1/convai/").split('/').collect();
        match (method.as_str(), segments.as_slice()) {
            ("GET", ["conversation", "get-signed-url"]) => signed_url(),
            ("GET", [collection]) => self.list(collection),
            ("POST", ["tools"] | ["agents", "create"]) => self.create(segments[0], body),
            ("GET", [collection, id]) => self.item(collection, id, None),
            ("PATCH", [collection, id]) => self.item(collection, id, Some(body)),
            _ => (StatusCode::NOT_FOUND, json!({"detail": "no route"})),
        }
    }
}

fn signed_url() -> Answer {
    (
        StatusCode::OK,
        json!({"signed_url": "wss://voice.example/agent"}),
    )
}
