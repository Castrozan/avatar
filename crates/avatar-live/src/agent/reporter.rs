use super::report::AgentReport;
use crate::boundaries::Courier;

pub(super) struct Reporter<'a, Crew: Courier> {
    pub(super) crew: &'a mut Crew,
    started: f64,
}

fn rounded_to_tenth(seconds: f64) -> f64 {
    format!("{seconds:.1}").parse().unwrap_or(seconds)
}

impl<'a, Crew: Courier> Reporter<'a, Crew> {
    pub(super) fn new(crew: &'a mut Crew) -> Self {
        Self {
            started: crew.now(),
            crew,
        }
    }

    pub(super) async fn report(&mut self, text: &str, is_final: bool) {
        let seconds = rounded_to_tenth(self.crew.now() - self.started);
        self.crew
            .report(AgentReport {
                text: text.to_owned(),
                seconds,
                is_final,
            })
            .await;
    }
}
