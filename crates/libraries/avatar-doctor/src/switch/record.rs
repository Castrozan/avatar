use std::cmp::Ordering;

use jiff::Timestamp;
use serde_json::{Map, Value};

use super::failure::LatencyError;
use crate::numeric::seconds_from_micros;

#[derive(Clone, Debug, PartialEq)]
pub struct Record(Map<String, Value>);

#[derive(Clone, Debug, PartialEq)]
pub struct Logged {
    pub at: f64,
    pub record: Record,
}

impl Record {
    #[must_use]
    pub fn parsed(line: &str) -> Option<Self> {
        match serde_json::from_str::<Value>(line) {
            Ok(Value::Object(fields)) if is_logged_line(line, &fields) => Some(Self(fields)),
            _ => None,
        }
    }

    #[must_use]
    pub fn text(&self, key: &str) -> Option<&str> {
        self.0.get(key)?.as_str()
    }

    #[must_use]
    pub fn number(&self, key: &str) -> Option<f64> {
        self.0.get(key)?.as_f64()
    }

    #[must_use]
    pub fn sequence(&self) -> Option<i64> {
        self.0.get("sequence")?.as_i64()
    }

    pub fn required_sequence(&self) -> Result<Option<i64>, LatencyError> {
        match self.0.get("sequence") {
            Some(_) => Ok(self.sequence()),
            None => Err(LatencyError::MissingField("sequence")),
        }
    }

    pub fn required_number(&self, key: &'static str) -> Result<f64, LatencyError> {
        match self.0.get(key) {
            None => Err(LatencyError::MissingField(key)),
            Some(value) => value.as_f64().ok_or(LatencyError::NotNumeric(key)),
        }
    }

    #[must_use]
    pub fn is_text(&self, key: &str, expected: &str) -> bool {
        self.text(key) == Some(expected)
    }

    #[must_use]
    pub fn has(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    #[must_use]
    pub fn is_recorded(&self, key: &str) -> bool {
        self.0.get(key).is_some_and(|value| !value.is_null())
    }

    pub fn timestamp(&self) -> Result<f64, LatencyError> {
        match self.0.get("timestamp") {
            Some(Value::String(text)) => parse_timestamp(text),
            other => Err(LatencyError::Timestamp(
                other.map(ToString::to_string).unwrap_or_default(),
            )),
        }
    }
}

fn is_logged_line(line: &str, fields: &Map<String, Value>) -> bool {
    line.starts_with('{') && fields.contains_key("timestamp")
}

fn parse_timestamp(text: &str) -> Result<f64, LatencyError> {
    let rejected = || LatencyError::Timestamp(text.to_owned());
    let parsed: Timestamp = text.parse().map_err(|_| rejected())?;
    seconds_from_micros(parsed.as_microsecond()).ok_or_else(rejected)
}

pub fn logged_records<'a>(
    lines: impl IntoIterator<Item = &'a str>,
) -> Result<Vec<Logged>, LatencyError> {
    let mut logged = lines
        .into_iter()
        .filter_map(Record::parsed)
        .map(logged_at)
        .collect::<Result<Vec<Logged>, LatencyError>>()?;
    logged.sort_by(by_time);
    Ok(logged)
}

fn logged_at(record: Record) -> Result<Logged, LatencyError> {
    Ok(Logged {
        at: record.timestamp()?,
        record,
    })
}

fn by_time(earlier: &Logged, later: &Logged) -> Ordering {
    earlier.at.partial_cmp(&later.at).unwrap_or(Ordering::Equal)
}
