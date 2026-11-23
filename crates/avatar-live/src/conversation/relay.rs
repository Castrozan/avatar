use std::future::Future;

use avatar_transcript::json::python_float;

use crate::agent::report::AgentReport;
use crate::boundaries::{Clock, Console, ReportSink, VoiceDelivery};
use crate::events::TaskRequest;

pub struct Podium<Voice: VoiceDelivery, Screen: Console> {
    pub voice: Voice,
    pub screen: Screen,
}

pub(super) struct Relay<'a, Pace: Clock, Voice: VoiceDelivery, Screen: Console> {
    pace: &'a mut Pace,
    podium: &'a mut Podium<Voice, Screen>,
    task_id: String,
    servant: String,
}

impl<'a, Pace: Clock, Voice: VoiceDelivery, Screen: Console> Relay<'a, Pace, Voice, Screen> {
    pub(super) fn new(
        pace: &'a mut Pace,
        podium: &'a mut Podium<Voice, Screen>,
        task: &TaskRequest,
        servant: &str,
    ) -> Self {
        Self {
            pace,
            podium,
            task_id: task.task_id.clone(),
            servant: servant.to_owned(),
        }
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
        self.podium
            .voice
            .deliver(&self.task_id, &report.text, report.is_final)
            .await;
        let seconds = python_float(report.seconds);
        self.podium
            .screen
            .print(&format!("{}, {seconds} s: {}", self.servant, report.text));
    }
}
