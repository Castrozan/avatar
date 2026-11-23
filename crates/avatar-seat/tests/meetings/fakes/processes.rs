use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use avatar_seat::edges::{CommandOutcome, EdgeFailure, Processes};

use serde_json::Value;

use super::scripts::RunScript;

pub(crate) type Log = Rc<RefCell<Vec<Value>>>;
pub(crate) type Hook = Box<dyn Fn(&[String])>;

pub(crate) struct ScriptedProcesses {
    runs: RefCell<VecDeque<RunScript>>,
    log: Log,
    pub(crate) hook: Hook,
}

impl ScriptedProcesses {
    pub(crate) fn new(runs: &[Value], log: &Log, os_message: &str) -> Self {
        let scripts = runs
            .iter()
            .map(|entry| RunScript::from_value(entry, os_message));
        Self {
            runs: RefCell::new(scripts.collect()),
            log: log.clone(),
            hook: Box::new(|_command| ()),
        }
    }
}

fn call_line(command: &[String], seconds: u64, capture: bool) -> Value {
    Value::String(format!(
        "$ {}|{seconds}|{}",
        command.join(" "),
        u8::from(capture)
    ))
}

fn normalised_call(call: &Value) -> Value {
    let line = call.as_str().filter(|text| text.starts_with("$ "));
    let flags = line.and_then(|text| text.rsplit_once('|'));
    let rewritten = flags.map(|(head, tail)| Value::String(format!("{head}|{}", &tail[1..])));
    rewritten.unwrap_or_else(|| call.clone())
}

pub(crate) fn assert_calls(log: &Log, expected: &Value) {
    let wanted = expected.as_array().cloned().unwrap_or_default();
    let normalised: Vec<Value> = wanted.iter().map(normalised_call).collect();
    assert_eq!(log.borrow().clone(), normalised);
}

fn notify(hook: &Hook, command: &[String], result: &Result<CommandOutcome, EdgeFailure>) {
    if result.is_ok() {
        hook(command);
    }
}

impl Processes for ScriptedProcesses {
    fn run(
        &self,
        command: &[String],
        seconds: u64,
        capture: bool,
    ) -> Result<CommandOutcome, EdgeFailure> {
        self.log
            .borrow_mut()
            .push(call_line(command, seconds, capture));
        let script = self.runs.borrow_mut().pop_front().unwrap_or_default();
        let result = script.into_outcome(command, seconds);
        notify(&self.hook, command, &result);
        result
    }

    fn spawn_detached(&self, _command: &[String]) -> Result<(), EdgeFailure> {
        Ok(())
    }
}
