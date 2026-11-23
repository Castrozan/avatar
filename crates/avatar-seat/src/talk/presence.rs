use regex::Regex;

use crate::edges::EdgeFailure;
use crate::local::LocalSeat;

const PRESENT_WITHIN_NANOSECONDS: u128 = 300 * 1_000_000_000;
const IDLE_PATTERN: &str = r#""HIDIdleTime" = (\d+)"#;
const IOREG_TIMEOUT_SECONDS: u64 = 10;

pub(crate) fn user_is_here(local: &LocalSeat<'_>) -> Result<bool, EdgeFailure> {
    let command = ["/usr/sbin/ioreg", "-c", "IOHIDSystem"].map(str::to_owned);
    let report = local.processes.run(&command, IOREG_TIMEOUT_SECONDS, true)?;
    let idle = idle_nanoseconds(&report.stdout);
    Ok(idle.is_some_and(|nanoseconds| nanoseconds < PRESENT_WITHIN_NANOSECONDS))
}

fn idle_nanoseconds(report: &str) -> Option<u128> {
    let pattern = Regex::new(IDLE_PATTERN).ok()?;
    let digits = pattern.captures(report)?.get(1)?;
    digits.as_str().parse().ok()
}
