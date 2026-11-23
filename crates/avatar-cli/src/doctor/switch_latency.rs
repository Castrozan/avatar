use std::io::{self, Write};
use std::process::ExitCode;

use avatar_doctor::switch::failure::LatencyError;
use avatar_doctor::switch::moment::Switch;
use avatar_doctor::switch::report::report;
use avatar_doctor::switch::report::window::{window_title, within};
use avatar_doctor::switch::store::history_file::history_beside;
use avatar_doctor::switch::store::recorded_switches;
use avatar_doctor::switch::store::wake_log::RotatedWakeLog;
use clap::Args;
use jiff::civil::{Date, DateTime, Time};
use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};

use crate::support::wake_log::log_path;

#[derive(Args)]
pub(crate) struct SwitchLatencyOptions {
    #[arg(long, value_parser = moment, help = "Show switches from this ISO moment; default the start of today")]
    since: Option<f64>,
    #[arg(long, value_parser = moment, help = "Show switches before this ISO moment; default no end")]
    until: Option<f64>,
    #[arg(
        long,
        help = "only keep the switches in the wake log in the lasting history"
    )]
    record: bool,
}

fn seconds_of(moment: &Timestamp) -> f64 {
    moment.as_duration().as_secs_f64()
}

fn local(text: &str) -> Result<Timestamp, jiff::Error> {
    let zone = TimeZone::system();
    let civil = text.parse::<DateTime>().or_else(|_| {
        text.parse::<Date>()
            .map(|date| date.to_datetime(Time::midnight()))
    })?;
    Ok(civil.to_zoned(zone)?.timestamp())
}

fn moment(text: &str) -> Result<f64, String> {
    let parsed = text.parse::<Timestamp>().or_else(|_| local(text));
    parsed
        .map(|found| seconds_of(&found))
        .map_err(|error| error.to_string())
}

fn start_of_today() -> Result<f64, LatencyError> {
    let now = Zoned::now();
    let midnight = now
        .start_of_day()
        .map_err(|error| LatencyError::Timestamp(error.to_string()))?;
    Ok(seconds_of(&midnight.timestamp()))
}

fn failure(error: &LatencyError) -> io::Error {
    io::Error::other(error.to_string())
}

fn recorded() -> Result<Vec<Switch>, LatencyError> {
    let path = log_path();
    recorded_switches(&RotatedWakeLog::at(&path), &history_beside(&path))
}

fn rendered(options: &SwitchLatencyOptions, switches: &[Switch]) -> Result<String, LatencyError> {
    let since = options.since.map_or_else(start_of_today, Ok)?;
    let until = options.until.unwrap_or(f64::INFINITY);
    let zone = TimeZone::system();
    let title = window_title(since, until, &zone)?;
    Ok(format!(
        "{title}\n{}",
        report(&within(switches, since, until), &zone)?
    ))
}

pub(crate) fn run(options: &SwitchLatencyOptions) -> io::Result<ExitCode> {
    let switches = recorded().map_err(|error| failure(&error))?;
    if options.record {
        return Ok(ExitCode::SUCCESS);
    }
    let text = rendered(options, &switches).map_err(|error| failure(&error))?;
    writeln!(io::stdout().lock(), "{text}")?;
    Ok(ExitCode::SUCCESS)
}
