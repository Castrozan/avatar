use avatar_conversation::talk::ports::Files;
use serde_json::{Value, json};

use super::script::at_of;
use super::{FileState, START, WALL, World};

fn initial_text(spec: &Value) -> (String, f64) {
    let age = spec["age"].as_f64().unwrap_or(0.0);
    let text = spec.as_str().or_else(|| spec["content"].as_str());
    (text.unwrap_or_default().to_owned(), age)
}

fn initial_state(started: f64, spec: &Value) -> FileState {
    let (content, age) = initial_text(spec);
    FileState {
        content: Some(content),
        mtime: WALL + started - START - age,
    }
}

fn after_event(event: &Value, existing: Option<String>) -> Option<String> {
    let addition = event["content"].as_str().map(str::to_owned);
    match (event["append"] == Value::Bool(true), existing, addition) {
        (true, Some(before), Some(after)) => Some(before + &after),
        (_, _, after) => after,
    }
}

fn apply_all(world: &mut World, events: &[Value]) {
    for event in events {
        world.apply(event);
    }
}

impl World {
    pub(super) fn load_initial_files(&mut self) {
        let initial = self
            .scenario_field("files")
            .as_object()
            .cloned()
            .unwrap_or_default();
        let started = self.now;
        let states = initial
            .iter()
            .map(|(name, spec)| (name.clone(), initial_state(started, spec)));
        self.files.extend(states);
    }

    fn apply(&mut self, event: &Value) {
        let name = event["name"].as_str().unwrap_or_default();
        let existing = self.files.get(name).and_then(|state| state.content.clone());
        let content = after_event(event, existing);
        let mtime = WALL + at_of(event);
        self.files
            .insert(name.to_owned(), FileState { content, mtime });
    }

    fn sync(&mut self) {
        let due = self
            .events
            .partition_point(|event| at_of(event) <= self.now - START);
        let applied: Vec<Value> = self.events.drain(..due).collect();
        apply_all(self, &applied);
    }

    fn entry(&mut self, name: &str) -> Option<&FileState> {
        self.sync();
        self.files.get(name).filter(|state| state.content.is_some())
    }

    pub(super) fn wall_time(&self) -> f64 {
        WALL + (self.now - START)
    }

    fn replace(&mut self, name: &str, content: String) {
        let mtime = self.wall_time();
        self.files.insert(
            name.to_owned(),
            FileState {
                content: Some(content),
                mtime,
            },
        );
    }
}

impl Files for World {
    fn exists(&mut self, name: &str) -> bool {
        self.entry(name).is_some()
    }

    fn read(&mut self, name: &str) -> Option<String> {
        self.entry(name).and_then(|state| state.content.clone())
    }

    fn modified_at(&mut self, name: &str) -> Option<f64> {
        self.entry(name).map(|state| state.mtime)
    }

    fn write(&mut self, name: &str, content: &str) {
        self.stamp(json!({"call": "write", "name": name, "content": content}));
        self.replace(name, content.to_owned());
    }

    fn touch(&mut self, name: &str) {
        self.stamp(json!({"call": "touch", "name": name}));
        let content = self.read(name).unwrap_or_default();
        self.replace(name, content);
    }
}
