use std::future::Future;

use avatar_floor::addressee::Addressee;

use crate::agent::report::{AgentReport, AgentRequest};
use crate::spoken_turns::SpokenTurn;

pub trait Speaker {
    fn play(&mut self, pcm: &[u8]);
    fn buffered_seconds(&self) -> f64;
    fn clear(&mut self);
}

pub trait TurnLog {
    fn append(&mut self, turn: &SpokenTurn);
}

pub trait Console {
    fn print(&mut self, line: &str);
}

pub trait Clock {
    fn now(&self) -> f64;
    fn sleep(&mut self, seconds: f64) -> impl Future<Output = ()>;
}

pub trait HolderLookup {
    fn current(&mut self) -> Option<Addressee>;
}

pub trait ReportSink {
    fn report(&mut self, report: AgentReport) -> impl Future<Output = ()>;
}

pub trait VoiceDelivery {
    fn deliver(&mut self, task_id: &str, result: &str, is_final: bool) -> impl Future<Output = ()>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandOutput {
    pub code: Option<i32>,
    pub output: String,
}

pub trait AgentTransport {
    fn run(
        &mut self,
        holder: &Addressee,
        arguments: &[String],
    ) -> impl Future<Output = CommandOutput>;
}

pub trait Courier: Clock + ReportSink {}

impl<Both: Clock + ReportSink> Courier for Both {}

pub trait AgentAsker {
    fn ask(
        &mut self,
        request: &AgentRequest,
        courier: &mut impl Courier,
    ) -> impl Future<Output = ()>;

    fn notify(
        &mut self,
        request: &AgentRequest,
        reason: &str,
    ) -> impl Future<Output = Result<(), String>>;
}
