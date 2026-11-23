pub mod clock;
pub mod format;
pub mod lines;
pub mod span;
pub mod stage;
pub mod summary;
pub mod window;

use jiff::tz::TimeZone;

use super::failure::LatencyError;
use super::moment::Switch;

pub fn report(switches: &[Switch], zone: &TimeZone) -> Result<String, LatencyError> {
    let mut rendered = Vec::new();
    for switch in switches {
        rendered.extend(lines::switch_lines(switch, zone)?);
        rendered.push(String::new());
    }
    rendered.extend(summary::summary_lines(switches));
    Ok(rendered.join("\n"))
}
