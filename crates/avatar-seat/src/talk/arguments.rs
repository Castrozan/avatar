use serde_json::Value;

use super::REMOTE_SEAT;

#[must_use]
pub fn asks_for_the_remote_seat(arguments: &[String]) -> bool {
    arguments
        .windows(2)
        .any(|pair| pair[0] == "--seat" && pair[1] == REMOTE_SEAT)
}

#[must_use]
pub fn opens_a_seated_conversation(arguments: &[String]) -> bool {
    arguments.first().is_some_and(|verb| verb == "open")
        && !arguments.iter().any(|word| word == "--seat")
}

#[must_use]
pub fn with_the_remote_seat(arguments: &[String]) -> Vec<String> {
    let mut extended = arguments.to_vec();
    if !asks_for_the_remote_seat(arguments) {
        extended.extend(["--seat".to_owned(), REMOTE_SEAT.to_owned()]);
    }
    extended
}

#[must_use]
pub fn reported_seat(status_output: &str) -> Option<Value> {
    let last_line = status_output.trim().lines().next_back()?;
    let status: Value = serde_json::from_str(last_line).ok()?;
    status.get("seat").cloned()
}
