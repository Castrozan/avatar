use avatar_conversation::talk::ports::Files;
use serde_json::Value;

use super::{FileState, START, WALL, World, at_of};

impl World {
    pub(super) fn load_initial_files(&mut self) {
        let initial = self
            .scenario_field("files")
            .as_object()
            .cloned()
            .unwrap_or_default();
        for (name, spec) in initial {
            let (content, age) = match spec.as_str() {
                Some(text) => (text.to_owned(), 0.0),
                None => (
                    spec["content"].as_str().unwrap_or_default().to_owned(),
                    spec["age"].as_f64().unwrap_or(0.0),
                ),
            };
            let mtime = WALL + self.now - START - age;
            self.files.insert(
                name,
                FileState {
                    content: Some(content),
                    mtime,
                },
            );
        }
    }

    fn apply(&mut self, event: &Value) {
        let name = event["name"].as_str().unwrap_or_default().to_owned();
        let addition = event["content"].as_str().map(str::to_owned);
        let existing = self
            .files
            .get(&name)
            .and_then(|state| state.content.clone());
        let appended = event["append"] == Value::Bool(true);
        let content = match (appended, existing, addition) {
            (true, Some(before), Some(after)) => Some(before + &after),
            (_, _, after) => after,
        };
        self.files.insert(
            name,
            FileState {
                content,
                mtime: WALL + at_of(event),
            },
        );
    }

    fn sync(&mut self) {
        while self
            .events
            .first()
            .is_some_and(|event| at_of(event) <= self.now - START)
        {
            let event = self.events.remove(0);
            self.apply(&event);
        }
    }

    fn entry(&mut self, name: &str) -> Option<&FileState> {
        self.sync();
        self.files.get(name).filter(|state| state.content.is_some())
    }

    pub(super) fn wall_time(&self) -> f64 {
        WALL + (self.now - START)
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
        self.stamp(serde_json::json!({"call": "write", "name": name, "content": content}));
        let mtime = self.wall_time();
        self.files.insert(
            name.to_owned(),
            FileState {
                content: Some(content.to_owned()),
                mtime,
            },
        );
    }

    fn touch(&mut self, name: &str) {
        self.stamp(serde_json::json!({"call": "touch", "name": name}));
        let content = self
            .entry(name)
            .and_then(|state| state.content.clone())
            .unwrap_or_default();
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
