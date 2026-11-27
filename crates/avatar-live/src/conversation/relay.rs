use std::future::Future;

use super::ending::Ending;
use super::errand::Errand;
use crate::agent::report::AgentReport;
use crate::agent::reporter::elapsed_seconds;
use crate::boundaries::{Clock, Console, ReportSink, VoiceDelivery};

pub struct Podium<Voice: VoiceDelivery, Screen: Console> {
    pub voice: Voice,
    pub screen: Screen,
}

pub(super) struct Relay<'a, Pace: Clock, Voice: VoiceDelivery, Screen: Console> {
    pace: &'a mut Pace,
    podium: &'a mut Podium<Voice, Screen>,
    errand: &'a Errand,
    started: f64,
    failure: Option<String>,
}

impl<'a, Pace: Clock, Voice: VoiceDelivery, Screen: Console> Relay<'a, Pace, Voice, Screen> {
    pub(super) fn new(
        pace: &'a mut Pace,
        podium: &'a mut Podium<Voice, Screen>,
        errand: &'a Errand,
    ) -> Self {
        Self {
            started: pace.now(),
            pace,
            podium,
            errand,
            failure: None,
        }
    }

    pub(super) fn ending(&self, stopped: bool) -> Ending {
        let seconds = elapsed_seconds(self.started, self.pace.now());
        Ending::of(stopped, seconds, self.failure.clone())
    }
}

impl<Pace: Clock, Voice: VoiceDelivery, Screen: Console> Clock for Relay<'_, Pace, Voice, Screen> {
    fn now(&self) -> f64 {
        self.pace.now()
    }

    fn sleep(&mut self, seconds: f64) -> impl Future<Output = ()> {
        self.pace.sleep(seconds)
    }
}

impl<Pace: Clock, Voice: VoiceDelivery, Screen: Console> ReportSink
    for Relay<'_, Pace, Voice, Screen>
{
    async fn report(&mut self, report: AgentReport) {
        let voice = &mut self.podium.voice;
        voice
            .deliver(&self.errand.task_id, &report.text, report.kind.is_final())
            .await;
        self.podium.screen.print(&self.errand.report_line(&report));
        self.failure = report.kind.into_failure().or(self.failure.take());
    }
}
