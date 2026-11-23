use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};

use crate::numeric::whole_number;
use crate::switch::failure::LatencyError;

pub const UNRECORDED: &str = "-";

pub fn wall_clock(at: f64, zone: &TimeZone) -> Result<Zoned, LatencyError> {
    let (seconds, nanoseconds) = split_timestamp(at).ok_or(LatencyError::OutOfRange(at))?;
    let instant = Timestamp::new(seconds, nanoseconds).map_err(|_| LatencyError::OutOfRange(at))?;
    Ok(instant.to_zoned(zone.clone()))
}

pub fn clock_text(at: Option<f64>, zone: &TimeZone) -> Result<String, LatencyError> {
    let Some(at) = at else {
        return Ok(UNRECORDED.to_owned());
    };
    Ok(clock_face(&wall_clock(at, zone)?))
}

fn clock_face(local: &Zoned) -> String {
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        local.hour(),
        local.minute(),
        local.second(),
        local.millisecond()
    )
}

fn split_timestamp(at: f64) -> Option<(i64, i32)> {
    let whole = at.trunc();
    let (whole, micros) = carry(whole, ((at - whole) * 1e6).round_ties_even());
    let nanoseconds = i32::try_from(whole_number(micros)? * 1000).ok()?;
    Some((whole_number(whole)?, nanoseconds))
}

fn carry(whole: f64, micros: f64) -> (f64, f64) {
    if micros >= 1e6 {
        (whole + 1.0, micros - 1e6)
    } else if micros < 0.0 {
        (whole - 1.0, micros + 1e6)
    } else {
        (whole, micros)
    }
}
