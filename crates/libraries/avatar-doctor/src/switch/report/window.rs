use jiff::tz::TimeZone;

use super::super::failure::LatencyError;
use super::super::moment::Switch;
use super::clock::wall_clock;

#[must_use]
pub fn within(switches: &[Switch], since: f64, until: f64) -> Vec<Switch> {
    switches
        .iter()
        .filter(|switch| since <= switch.switched_at && switch.switched_at < until)
        .cloned()
        .collect()
}

pub fn window_title(since: f64, until: f64, zone: &TimeZone) -> Result<String, LatencyError> {
    let start = wall_clock(since, zone)?.strftime("%F %R").to_string();
    let end = if until.is_infinite() && until > 0.0 {
        "now".to_owned()
    } else {
        wall_clock(until, zone)?.strftime("%F %R").to_string()
    };
    Ok(format!("Switch round trips from {start} to {end}\n"))
}
