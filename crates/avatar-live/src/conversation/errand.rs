use avatar_floor::addressee::Addressee;
use avatar_transcript::json::python_float;

use crate::agent::brief::STOPPED_REASON;
use crate::agent::report::{AgentQuestion, AgentReport, AgentRequest, ReportKind};
use crate::events::TaskRequest;

pub(super) struct Errand {
    pub(super) request: AgentRequest,
    pub(super) task_id: String,
}

impl Errand {
    pub(super) fn new(task: &TaskRequest, holder: Addressee, number: u32) -> Self {
        let question = AgentQuestion {
            request: task.request.clone(),
            number,
        };
        Self {
            request: AgentRequest { holder, question },
            task_id: task.task_id.clone(),
        }
    }

    fn number(&self) -> u32 {
        self.request.question.number
    }

    fn servant(&self) -> &str {
        &self.request.holder.servant
    }

    pub(super) fn sent_line(&self) -> String {
        let (number, servant) = (self.number(), self.servant());
        let (pane, request) = (&self.request.holder.pane, &self.request.question.request);
        format!("task {number} for {servant} at {pane}: {request}")
    }

    pub(super) fn report_line(&self, report: &AgentReport) -> String {
        let seconds = python_float(report.seconds);
        match &report.kind {
            ReportKind::Progress => format!("{}, {seconds} s: {}", self.servant(), report.text),
            ReportKind::Answer => self.answered_line(&seconds, &report.text),
            ReportKind::Failure(reason) => self.failed_line(&seconds, reason),
        }
    }

    fn answered_line(&self, seconds: &str, answer: &str) -> String {
        let (number, servant) = (self.number(), self.servant());
        format!("task {number} answered by {servant} in {seconds} s: {answer}")
    }

    fn failed_line(&self, seconds: &str, reason: &str) -> String {
        let (number, servant) = (self.number(), self.servant());
        format!("task {number} failed for {servant} after {seconds} s: {reason}")
    }

    pub(super) fn unanswered_line(&self, seconds: f64) -> String {
        let (number, servant) = (self.number(), self.servant());
        let seconds = python_float(seconds);
        format!("task {number} unanswered for {servant} after {seconds} s: {STOPPED_REASON}")
    }

    pub(super) fn notice_line(&self, reason: &str, delivery: &Result<(), String>) -> String {
        let (number, servant) = (self.number(), self.servant());
        match delivery {
            Ok(()) => format!("task {number} notice sent to {servant}: {reason}"),
            Err(failure) => format!("task {number} notice not sent to {servant}: {failure}"),
        }
    }
}
