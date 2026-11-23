use std::cell::{Cell, RefCell};
use std::error::Error;
use std::io;
use std::rc::Rc;

use avatar_room_audio::mouth::{MouthLevels, VoiceLevelSink};
use avatar_room_audio::pcm::encode_samples;
use serde::Deserialize;

const FIXTURE: &str = include_str!("mouth/fixtures/mouth_levels.json");
const CONSTRUCTED_AT: f64 = 100.0;

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
enum Operation {
    Show { at: f64, samples: Vec<i16> },
    Deliver { at: f64 },
    Silence { at: f64 },
    Next { at: f64 },
    Close { at: f64 },
}

#[derive(Deserialize)]
struct MouthTrace {
    operations: Vec<Operation>,
    sent: Vec<f64>,
    next: Vec<Option<f64>>,
}

#[derive(Clone, Default)]
struct RecordingSink {
    messages: Rc<RefCell<Vec<String>>>,
    closed: Rc<Cell<bool>>,
}

impl VoiceLevelSink for RecordingSink {
    fn send_text(&mut self, text: &str) -> io::Result<()> {
        self.messages.borrow_mut().push(text.to_owned());
        Ok(())
    }

    fn close(&mut self) {
        self.closed.set(true);
    }
}

fn sent_levels(messages: &[String]) -> Vec<f64> {
    messages
        .iter()
        .filter_map(|message| serde_json::from_str::<serde_json::Value>(message).ok())
        .filter_map(|message| message.get("value").and_then(serde_json::Value::as_f64))
        .collect()
}

fn apply(
    levels: &mut MouthLevels<RecordingSink>,
    operation: &Operation,
    observed: &mut Vec<Option<f64>>,
) -> io::Result<()> {
    match operation {
        Operation::Show { at, samples } => levels.show(&encode_samples(samples), *at),
        Operation::Deliver { at } => levels.deliver_due(*at)?,
        Operation::Silence { at } => levels.silence(*at)?,
        Operation::Next { at } => observed.push(levels.seconds_until_next_delivery(*at)),
        Operation::Close { at } => levels.close(*at),
    }
    Ok(())
}

fn replay(trace: &MouthTrace) -> io::Result<(Vec<f64>, Vec<Option<f64>>)> {
    let sink = RecordingSink::default();
    let messages = Rc::clone(&sink.messages);
    let mut levels = MouthLevels::new(sink, CONSTRUCTED_AT);
    let mut observed = Vec::new();
    for operation in &trace.operations {
        apply(&mut levels, operation, &mut observed)?;
    }
    let recorded = messages.borrow().clone();
    Ok((sent_levels(&recorded), observed))
}

fn bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|value| value.to_bits()).collect()
}

fn optional_bits(values: &[Option<f64>]) -> Vec<Option<u64>> {
    values.iter().map(|value| value.map(f64::to_bits)).collect()
}

fn assert_trace_reproduced(trace: &MouthTrace) -> Result<(), io::Error> {
    let (sent, next) = replay(trace)?;
    assert_eq!(bits(&sent), bits(&trace.sent));
    assert_eq!(optional_bits(&next), optional_bits(&trace.next));
    Ok(())
}

#[test]
fn every_recorded_python_mouth_trace_is_reproduced_bit_for_bit() -> Result<(), Box<dyn Error>> {
    let traces: Vec<MouthTrace> = serde_json::from_str(FIXTURE)?;
    assert_eq!(traces.len(), 16);
    for trace in &traces {
        assert_trace_reproduced(trace)?;
    }
    Ok(())
}
