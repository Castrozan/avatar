use avatar_floor::addressee::Addressee;
use futures_util::future::OptionFuture;

use super::super::brief::{
    NO_ANSWER_IN_TIME, NO_ANSWER_REASON, could_not_finish, send_arguments, status_arguments,
};
use super::super::report::{AgentQuestion, ReportKind};
use super::super::reporter::Reporter;
use super::super::task::{Verdict, parse_task, verdict_in};
use super::super::text::python_lines;
use super::super::watch::Observation;
use super::polling::{Polling, poll_until_answered};
use crate::boundaries::{AgentTransport, CommandOutput, Courier};

pub(super) struct Dispatch<'a, Transport: AgentTransport, Crew: Courier> {
    pub(super) transport: &'a mut Transport,
    pub(super) reporter: Reporter<'a, Crew>,
    pub(super) holder: &'a Addressee,
}

fn exit_code_text(code: Option<i32>) -> String {
    code.map_or("None".to_owned(), |number| number.to_string())
}

pub(super) fn submission_of(sent: &CommandOutput) -> Result<String, String> {
    let lines = python_lines(sent.output.trim());
    match (lines.last(), sent.code) {
        (Some(last), Some(0)) => Ok(last.trim().to_owned()),
        (Some(last), _) => Err(last.clone()),
        (None, code) => Err(format!("exit {}", exit_code_text(code))),
    }
}

impl<Transport: AgentTransport, Crew: Courier> Dispatch<'_, Transport, Crew> {
    pub(super) async fn ask(mut self, question: &AgentQuestion) {
        let sent = self
            .transport
            .run(self.holder, &send_arguments(self.holder, question))
            .await;
        match submission_of(&sent) {
            Ok(task_id) => self.follow(task_id, question.answer_tag()).await,
            Err(failure) => self.report_unreachable(&failure).await,
        }
    }

    async fn report_unreachable(&mut self, failure: &str) {
        let kind = ReportKind::Failure(failure.to_owned());
        self.reporter.report(&could_not_finish(failure), kind).await;
    }

    async fn follow(&mut self, task_id: String, tag: String) {
        let mut polling = Polling::start(self, task_id, tag);
        let answered = poll_until_answered(&mut polling).await;
        self.report_timeout(answered).await;
    }

    async fn report_timeout(&mut self, answered: bool) {
        let kind = ReportKind::Failure(NO_ANSWER_REASON.to_owned());
        let timeout = (!answered).then(|| self.reporter.report(NO_ANSWER_IN_TIME, kind));
        OptionFuture::from(timeout).await;
    }

    pub(super) async fn status_verdict(&mut self, task_id: &str, tag: &str) -> Option<Verdict> {
        let status = self
            .transport
            .run(self.holder, &status_arguments(self.holder, task_id))
            .await;
        let output = Some(status.output).filter(|_| status.code == Some(0));
        let output = output.unwrap_or_default();
        verdict_in(&parse_task(&output), tag)
    }

    pub(super) async fn announce(&mut self, observation: Observation) -> bool {
        match observation.report() {
            None => false,
            Some((text, kind)) => self.say(&text, kind).await,
        }
    }

    async fn say(&mut self, text: &str, kind: ReportKind) -> bool {
        let is_final = kind.is_final();
        self.reporter.report(text, kind).await;
        is_final
    }
}
