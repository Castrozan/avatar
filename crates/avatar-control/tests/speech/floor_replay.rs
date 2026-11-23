use std::error::Error;

use avatar_control::speech::floor::{EnqueuedTurn, FloorEffect, FloorEvent, SpeechFloor, TurnId};
use serde::Deserialize;

use crate::names::{TestResult, phase_or_idle, status_named};

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
enum StepEvent {
    Enqueue { turn: TurnId, agent: bool },
    AgentOpened { turn: TurnId },
    Started { turn: TurnId },
    Finished { turn: TurnId },
    Failed { turn: TurnId },
    Interrupt,
}

#[derive(Deserialize)]
struct Step {
    event: StepEvent,
    effects: Vec<String>,
    phase: String,
}

#[derive(Deserialize)]
struct Scenario {
    name: String,
    steps: Vec<Step>,
}

fn enqueue_event(turn: TurnId, agent: bool) -> FloorEvent {
    FloorEvent::Enqueue(EnqueuedTurn {
        turn_id: turn,
        uses_agent_channel: agent,
    })
}

impl StepEvent {
    fn to_floor_event(&self) -> FloorEvent {
        match *self {
            Self::Enqueue { turn, agent } => enqueue_event(turn, agent),
            Self::AgentOpened { turn } => FloorEvent::AgentChannelOpened(turn),
            Self::Started { turn } => FloorEvent::SpeechStarted(turn),
            Self::Finished { turn } => FloorEvent::SpeechFinished(turn),
            Self::Failed { turn } => FloorEvent::SpeechFailed(turn),
            Self::Interrupt => FloorEvent::Interrupt,
        }
    }
}

fn settled(turn: &str, status: &str) -> Result<FloorEffect, Box<dyn Error>> {
    Ok(FloorEffect::Settle {
        turn_id: turn.parse()?,
        status: Some(status_named(status)?),
    })
}

fn parse_effect(text: &str) -> Result<FloorEffect, Box<dyn Error>> {
    let parts: Vec<&str> = text.split(':').collect();
    match parts.as_slice() {
        ["open", turn] => Ok(FloorEffect::OpenAgentChannel(turn.parse()?)),
        ["release", turn] => Ok(FloorEffect::ReleaseAgentChannel(turn.parse()?)),
        ["start", turn] => Ok(FloorEffect::StartSpeech(turn.parse()?)),
        ["stop", turn] => Ok(FloorEffect::StopPlayback(turn.parse()?)),
        ["settle", turn, status] => settled(turn, status),
        _ => Err(format!("unknown effect: {text}").into()),
    }
}

fn parse_effects(texts: &[String]) -> Result<Vec<FloorEffect>, Box<dyn Error>> {
    texts.iter().map(|text| parse_effect(text)).collect()
}

fn replay(scenario: &Scenario) -> TestResult {
    let mut floor = SpeechFloor::default();
    for step in &scenario.steps {
        let actual = floor.handle(step.event.to_floor_event())?;
        let expected = parse_effects(&step.effects)?;
        assert_eq!(actual, expected, "{} after {:?}", scenario.name, step.event);
        assert_eq!(floor.current_phase(), phase_or_idle(&step.phase)?);
    }
    Ok(())
}

#[test]
fn floor_scenarios_replay_the_javascript_floor() -> TestResult {
    let scenarios: Vec<Scenario> =
        serde_json::from_str(include_str!("fixtures/floor_scenarios.json"))?;
    assert_eq!(
        scenarios
            .iter()
            .map(|scenario| scenario.steps.len())
            .sum::<usize>(),
        56
    );
    scenarios.iter().try_for_each(replay)
}
