use std::future::Future;

use avatar_floor::addressee::Addressee;
use avatar_live::agent::report::AgentReport;
use avatar_live::boundaries::{AgentTransport, Clock, CommandOutput, ReportSink};
use serde_json::{Value, json};

use crate::support::{VirtualClock, array, text};

pub(super) struct ScriptedTransport {
    send: CommandOutput,
    statuses: Vec<CommandOutput>,
    pub(super) commands: Vec<Vec<String>>,
}

fn output_of(value: &Value) -> CommandOutput {
    let code = value["code"]
        .as_i64()
        .and_then(|number| i32::try_from(number).ok());
    CommandOutput {
        code,
        output: text(&value["output"]).to_owned(),
    }
}

fn command_of(arguments: &[String]) -> Vec<String> {
    std::iter::once("a2a".to_owned())
        .chain(arguments.iter().cloned())
        .collect()
}

impl ScriptedTransport {
    pub(super) fn new(case: &Value) -> Self {
        let statuses = array(&case["statuses"]).iter().map(output_of).collect();
        Self {
            send: output_of(&case["send"]),
            statuses,
            commands: Vec::new(),
        }
    }

    fn next_status(&mut self) -> CommandOutput {
        let advanced = (self.statuses.len() > 1).then(|| self.statuses.remove(0));
        let next = advanced.or_else(|| self.statuses.first().cloned());
        next.unwrap_or_else(|| output_of(&Value::Null))
    }
}

impl AgentTransport for ScriptedTransport {
    async fn run(&mut self, _holder: &Addressee, arguments: &[String]) -> CommandOutput {
        self.commands.push(command_of(arguments));
        match arguments.first().map(String::as_str) {
            Some("send") => self.send.clone(),
            _ => self.next_status(),
        }
    }
}

#[derive(Default)]
pub(super) struct Crew {
    pub(super) clock: VirtualClock,
    pub(super) reports: Vec<Value>,
}

impl Clock for Crew {
    fn now(&self) -> f64 {
        self.clock.now()
    }

    fn sleep(&mut self, seconds: f64) -> impl Future<Output = ()> {
        self.clock.sleep(seconds)
    }
}

impl ReportSink for Crew {
    async fn report(&mut self, report: AgentReport) {
        self.reports.push(
            json!({"text": report.text, "seconds": report.seconds, "final": report.is_final}),
        );
    }
}
