use std::time::Duration;

const RECORDER_PROGRAM: &str = "parec";
const REPORT_INTERVAL: Duration = Duration::from_millis(33);
const LISTEN_AGAIN: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub struct MeterSettings {
    pub program: String,
    pub source: String,
    pub listen_again: Duration,
    pub report_every: Duration,
}

impl MeterSettings {
    #[must_use]
    pub fn for_source(source: &str) -> Self {
        Self {
            program: RECORDER_PROGRAM.to_owned(),
            source: source.to_owned(),
            listen_again: LISTEN_AGAIN,
            report_every: REPORT_INTERVAL,
        }
    }
}
