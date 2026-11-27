use super::report::{AgentReport, ReportKind};
use crate::boundaries::Courier;

pub(super) struct Reporter<'a, Crew: Courier> {
    pub(super) crew: &'a mut Crew,
    started: f64,
}

fn rounded_to_tenth(seconds: f64) -> f64 {
    format!("{seconds:.1}").parse().unwrap_or(seconds)
}

#[must_use]
pub fn elapsed_seconds(started: f64, now: f64) -> f64 {
    rounded_to_tenth(now - started)
}

impl<'a, Crew: Courier> Reporter<'a, Crew> {
    pub(super) fn new(crew: &'a mut Crew) -> Self {
        Self {
            started: crew.now(),
            crew,
        }
    }

    pub(super) async fn report(&mut self, text: &str, kind: ReportKind) {
        let seconds = elapsed_seconds(self.started, self.crew.now());
        self.crew
            .report(AgentReport {
                text: text.to_owned(),
                seconds,
                kind,
            })
            .await;
    }
}
