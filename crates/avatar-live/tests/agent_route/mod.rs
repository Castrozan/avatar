mod fakes;

use avatar_live::agent::report::{AgentQuestion, AgentRequest};
use avatar_live::agent::route::AgentRoute;
use avatar_live::boundaries::AgentAsker;
use serde_json::{Value, json};

use self::fakes::{Crew, ScriptedTransport};
use crate::support::{Outcome, addressee, block_on, number, same, text, try_cases};

const FIXTURE: &str = include_str!("fixtures/agent_route.json");

fn question_of(case: &Value) -> AgentQuestion {
    let number = case["number"]
        .as_u64()
        .map_or(0, |value| u32::try_from(value).unwrap_or(0));
    AgentQuestion {
        request: text(&case["request"]).to_owned(),
        number,
    }
}

fn request_of(case: &Value) -> AgentRequest {
    AgentRequest {
        holder: addressee("wW:pQZ", "Suzuka"),
        question: question_of(case),
    }
}

async fn replay(case: &Value) -> (Value, Value, f64) {
    let mut route = AgentRoute::new(ScriptedTransport::new(case));
    let mut crew = Crew::default();
    route.ask(&request_of(case), &mut crew).await;
    (
        json!(crew.reports),
        json!(route.into_transport().commands),
        crew.clock.now,
    )
}

fn check(case: &Value) -> Outcome {
    let (reports, commands, now) = block_on(replay(case))?;
    same(&reports, &case["reports"], case);
    same(&commands, &case["commands"], case);
    assert_eq!(now.to_bits(), number(&case["clock"]).to_bits());
    Ok(())
}

#[test]
fn every_recorded_python_route_is_reproduced() -> Outcome {
    try_cases(FIXTURE, 15, check)
}
