use avatar_floor::addressee::Addressee;
use avatar_live::agent::report::AgentReport;
use avatar_live::conversation::{HandOffDesk, Podium};
use avatar_live::events::TaskRequest;
use serde_json::{Value, json};

use super::fakes::{Recorder, Screen};
use super::scripted::{Holder, ScriptedAsker, Voice};
use crate::support::{
    Outcome, VirtualClock, addressee, array, block_on, number, same, text, try_cases,
};

const FIXTURE: &str = include_str!("fixtures/hand_off.json");

type Desk = HandOffDesk<Holder, ScriptedAsker, Voice, Screen>;

fn holder_of(spec: &Value) -> Option<Addressee> {
    spec["pane"]
        .as_str()
        .map(|pane| addressee(pane, text(&spec["servant"])))
}

fn report_of(spec: &Value) -> AgentReport {
    AgentReport {
        text: text(&spec[0]).to_owned(),
        seconds: number(&spec[1]),
        is_final: spec[2] == json!(true),
    }
}

fn task_of(spec: &Value) -> TaskRequest {
    TaskRequest {
        task_id: text(&spec[0]).to_owned(),
        request: text(&spec[1]).to_owned(),
    }
}

fn route_for(case: &Value) -> ScriptedAsker {
    ScriptedAsker(
        array(&case["reports"]).iter().map(report_of).collect(),
        Vec::new(),
    )
}

fn podium_for(recorder: &Recorder) -> Podium<Voice, Screen> {
    Podium {
        voice: Voice(Vec::new()),
        screen: Screen(recorder.clone()),
    }
}

fn desk_for(case: &Value, recorder: &Recorder) -> Desk {
    let holders = Holder(holder_of(&case["holder"]));
    HandOffDesk {
        holders,
        route: route_for(case),
        podium: podium_for(recorder),
        answer_number: 0,
    }
}

async fn run(desk: &mut Desk, case: &Value) {
    for task in array(&case["tasks"]) {
        desk.hand_off(&task_of(task), &mut VirtualClock::default())
            .await;
    }
}

fn matches_python(desk: &Desk, recorder: &Recorder, case: &Value) {
    same(&json!(desk.podium.voice.0), &case["delivered"], case);
    same(&json!(desk.route.1), &case["asked"], case);
    same(
        &json!(recorder.printed.borrow().clone()),
        &case["printed"],
        case,
    );
}

fn check(case: &Value) -> Outcome {
    let recorder = Recorder::default();
    let mut desk = desk_for(case, &recorder);
    block_on(run(&mut desk, case))?;
    matches_python(&desk, &recorder, case);
    Ok(())
}

#[test]
fn every_recorded_python_hand_off_is_reproduced() -> Outcome {
    try_cases(FIXTURE, 7, check)
}
