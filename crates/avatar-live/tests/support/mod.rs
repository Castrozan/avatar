mod events;

use std::error::Error;
use std::future::Future;

use avatar_floor::addressee::Addressee;
use avatar_live::boundaries::Clock;
use avatar_live::events::LiveEvent;
use avatar_transcript::json::{Json, encode};
use serde_json::Value;

pub(crate) use self::events::{event_from, event_json};

pub(crate) type Outcome = Result<(), Box<dyn Error>>;

pub(crate) fn cases(fixture: &str, count: usize) -> Result<Vec<Value>, Box<dyn Error>> {
    let parsed: Vec<Value> = serde_json::from_str(fixture)?;
    assert_eq!(parsed.len(), count);
    Ok(parsed)
}

pub(crate) fn run_cases(fixture: &str, count: usize, check: fn(&Value)) -> Outcome {
    cases(fixture, count)?.iter().for_each(check);
    Ok(())
}

pub(crate) fn try_cases(fixture: &str, count: usize, check: fn(&Value) -> Outcome) -> Outcome {
    cases(fixture, count)?.iter().try_for_each(check)
}

pub(crate) fn addressee(pane: &str, servant: &str) -> Addressee {
    Addressee {
        pane: pane.to_owned(),
        servant: servant.to_owned(),
        heard_through: 0,
        host: None,
    }
}

pub(crate) fn plain(value: &Json) -> Result<Value, serde_json::Error> {
    serde_json::from_str(&encode(value))
}

pub(crate) fn array(value: &Value) -> &[Value] {
    value.as_array().map(Vec::as_slice).unwrap_or_default()
}

pub(crate) fn text(value: &Value) -> &str {
    value.as_str().unwrap_or_default()
}

pub(crate) fn number(value: &Value) -> f64 {
    value.as_f64().unwrap_or_default()
}

pub(crate) fn bytes(value: &Value) -> Vec<u8> {
    let numbers = array(value).iter().filter_map(Value::as_u64);
    numbers.filter_map(|byte| u8::try_from(byte).ok()).collect()
}

pub(crate) fn events_json(events: &[LiveEvent]) -> Vec<Value> {
    events.iter().map(event_json).collect()
}

pub(crate) fn same(actual: &Value, expected: &Value, case: &Value) {
    assert_eq!(actual, expected, "{}", text(&case["name"]));
}

pub(crate) fn block_on<Output>(
    future: impl Future<Output = Output>,
) -> Result<Output, std::io::Error> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    Ok(runtime.block_on(future))
}

#[derive(Default)]
pub(crate) struct VirtualClock {
    pub(crate) now: f64,
}

impl Clock for VirtualClock {
    fn now(&self) -> f64 {
        self.now
    }

    async fn sleep(&mut self, seconds: f64) {
        self.now += seconds;
    }
}
