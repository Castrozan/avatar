use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use avatar_audio_graph::errors::AudioGraphError;
use avatar_audio_graph::graph::AudioGraph;
use avatar_audio_graph::pulse::control::{PulseControl, PulseError};
use avatar_audio_graph::state::record::AudioState;

use super::fixture::{CallRecord, StepSpec};
use super::probe::ProbeStore;
use super::requests::request_for;

pub(crate) type Script = Rc<RefCell<ReplayScript>>;
pub(crate) type Probe = Rc<RefCell<Option<AudioState>>>;
type Expectation = (Vec<String>, Result<String, String>);

#[derive(Default)]
pub(crate) struct ReplayScript {
    expected: VecDeque<Expectation>,
    mismatches: Vec<String>,
}

pub(crate) fn load_script(
    script: &Script,
    calls: &[CallRecord],
    outputs: &[String],
) -> Result<(), String> {
    let mut borrowed = script.borrow_mut();
    borrowed.expected = calls
        .iter()
        .map(|call| expected_reply(call, outputs))
        .collect::<Result<_, _>>()?;
    borrowed.mismatches.clear();
    Ok(())
}

pub(crate) fn script_verdict(script: &Script) -> Result<(), String> {
    let borrowed = script.borrow();
    if borrowed.mismatches.is_empty() && borrowed.expected.is_empty() {
        return Ok(());
    }
    Err(format!(
        "mismatches {:?}, unconsumed {}",
        borrowed.mismatches,
        borrowed.expected.len()
    ))
}

fn expected_reply(call: &CallRecord, outputs: &[String]) -> Result<Expectation, String> {
    let reply = match (call.output, &call.error) {
        (Some(index), _) => Ok(outputs
            .get(index)
            .cloned()
            .ok_or("output index out of range")?),
        (None, Some(message)) => Err(message.clone()),
        (None, None) => return Err("call has neither output nor error".to_owned()),
    };
    Ok((call.argv.clone(), reply))
}

fn answer(script: &mut ReplayScript, arguments: &[&str]) -> Result<String, PulseError> {
    let Some((argv, reply)) = script.expected.pop_front() else {
        script.mismatches.push(format!("unexpected {arguments:?}"));
        return Err(PulseError::new("unexpected call"));
    };
    check_arguments(&mut script.mismatches, arguments, &argv)?;
    reply.map_err(PulseError::new)
}

fn check_arguments(
    mismatches: &mut Vec<String>,
    arguments: &[&str],
    argv: &[String],
) -> Result<(), PulseError> {
    let matches = arguments
        .iter()
        .copied()
        .eq(argv.iter().map(String::as_str));
    if !matches {
        mismatches.push(format!("got {arguments:?} expected {argv:?}"));
    }
    matches
        .then_some(())
        .ok_or_else(|| PulseError::new("argv mismatch"))
}

pub(crate) struct ScriptedPulse {
    script: Script,
}

impl ScriptedPulse {
    pub(crate) fn new(script: Script) -> Self {
        Self { script }
    }
}

impl PulseControl for ScriptedPulse {
    fn run(&mut self, arguments: &[&str]) -> Result<String, PulseError> {
        answer(&mut self.script.borrow_mut(), arguments)
    }
}

pub(crate) fn perform_step(
    graph: &mut AudioGraph<ScriptedPulse, ProbeStore>,
    step: &StepSpec,
) -> Result<(), String> {
    let outcome: Result<(), AudioGraphError> = match step.action.as_str() {
        "release" => graph.release_vanished_microphone(),
        _ => graph.perform(request_for(step)?),
    };
    outcome.map_err(|error| error.to_string())
}
