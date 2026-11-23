use std::error::Error;

use avatar_control::speech::chunk_sequence::{SequenceEffect, SequenceEvent, SpeechChunkSequence};
use serde::Deserialize;

use crate::names::TestResult;

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
enum StepEvent {
    Begin { count: usize },
    Ready { index: usize, holds: Option<bool> },
    Played { index: usize },
    Stop,
}

#[derive(Deserialize)]
struct Step {
    event: StepEvent,
    effects: Vec<String>,
}

#[derive(Deserialize)]
struct Scenario {
    name: String,
    steps: Vec<Step>,
}

impl StepEvent {
    fn to_sequence_event(&self) -> Result<SequenceEvent, Box<dyn Error>> {
        match *self {
            Self::Begin { .. } => Err("begin is not a sequence event".into()),
            Self::Ready { index: 0, holds } => Ok(SequenceEvent::FirstChunkReady {
                holds_floor: holds.ok_or("first chunk without holds")?,
            }),
            Self::Ready { index, .. } => Ok(SequenceEvent::ChunkReady(index)),
            Self::Played { index } => Ok(SequenceEvent::ChunkPlayed(index)),
            Self::Stop => Ok(SequenceEvent::Stop),
        }
    }
}

fn parse_effect(text: &str) -> Result<SequenceEffect, Box<dyn Error>> {
    let parts: Vec<&str> = text.split(':').collect();
    match parts.as_slice() {
        ["synth", index] => Ok(SequenceEffect::Synthesize(index.parse()?)),
        ["play", index] => Ok(SequenceEffect::Play(index.parse()?)),
        ["stop"] => Ok(SequenceEffect::StopPlayback),
        ["done"] => Ok(SequenceEffect::Finished),
        _ => Err(format!("unknown effect: {text}").into()),
    }
}

fn parse_effects(texts: &[String]) -> Result<Vec<SequenceEffect>, Box<dyn Error>> {
    texts.iter().map(|text| parse_effect(text)).collect()
}

fn drive(
    sequence: &mut Option<SpeechChunkSequence>,
    event: &StepEvent,
) -> Result<Vec<SequenceEffect>, Box<dyn Error>> {
    if let StepEvent::Begin { count } = event {
        let (created, effects) = SpeechChunkSequence::begin(*count);
        *sequence = Some(created);
        return Ok(effects);
    }
    let active = sequence.as_mut().ok_or("event before begin")?;
    Ok(active.handle(event.to_sequence_event()?))
}

fn replay(scenario: &Scenario) -> TestResult {
    let mut sequence = None;
    for step in &scenario.steps {
        let actual = drive(&mut sequence, &step.event)?;
        let expected = parse_effects(&step.effects)?;
        assert_eq!(actual, expected, "{} at {:?}", scenario.name, step.event);
    }
    Ok(())
}

#[test]
fn chunk_scenarios_replay_the_javascript_sequence() -> TestResult {
    let scenarios: Vec<Scenario> =
        serde_json::from_str(include_str!("fixtures/chunk_scenarios.json"))?;
    assert_eq!(
        scenarios
            .iter()
            .map(|scenario| scenario.steps.len())
            .sum::<usize>(),
        51
    );
    scenarios.iter().try_for_each(replay)
}
