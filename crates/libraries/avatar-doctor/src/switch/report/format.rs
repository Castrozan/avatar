use std::fmt::Display;

use super::clock::UNRECORDED;

#[must_use]
pub fn seconds(duration: Option<f64>) -> String {
    duration.map_or_else(|| UNRECORDED.to_owned(), |value| format!("{value:.3} s"))
}

#[must_use]
pub fn shown<Value: Display>(value: Option<&Value>) -> String {
    value.map_or_else(|| "None".to_owned(), ToString::to_string)
}
