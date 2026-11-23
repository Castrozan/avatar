pub mod history;
pub mod history_file;
pub mod wake_log;

use crate::switch::failure::LatencyError;
use crate::switch::moment::{Switch, switches_in};
use crate::switch::record::logged_records;
use history::{SwitchHistory, merged};
use wake_log::WakeLog;

pub fn recorded_switches(
    wake_log: &impl WakeLog,
    history: &impl SwitchHistory,
) -> Result<Vec<Switch>, LatencyError> {
    let lines = wake_log.lines()?;
    let fresh = switches_in(&logged_records(lines.iter().map(String::as_str))?)?;
    let switches = merged(history.remembered()?, fresh);
    history.keep(&switches)?;
    Ok(switches)
}
