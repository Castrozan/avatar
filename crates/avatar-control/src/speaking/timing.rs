use serde_json::{Map, Value};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpeechTiming {
    pub received: u64,
    pub synthesis_started: Option<u64>,
    pub synthesized: Option<u64>,
    pub playing: Option<u64>,
}

impl SpeechTiming {
    #[must_use]
    pub const fn received_at(received: u64) -> Self {
        Self {
            received,
            synthesis_started: None,
            synthesized: None,
            playing: None,
        }
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        let mut fields = Map::new();
        fields.insert("received".to_owned(), Value::from(self.received));
        insert_present(&mut fields, "synthesisStarted", self.synthesis_started);
        insert_present(&mut fields, "synthesized", self.synthesized);
        insert_present(&mut fields, "playing", self.playing);
        Value::Object(fields)
    }
}

fn insert_present(fields: &mut Map<String, Value>, key: &str, instant: Option<u64>) {
    if let Some(instant) = instant {
        fields.insert(key.to_owned(), Value::from(instant));
    }
}
