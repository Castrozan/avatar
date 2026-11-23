use avatar_live::spoken_turns::{Party, SpokenTurn, TurnAssembler};
use proptest::prelude::*;
use serde_json::{Value, json};

use crate::support::{Outcome, array, number, run_cases, same, text};

const FIXTURE: &str = include_str!("fixtures/spoken_turns.json");

fn party_of(name: &str) -> Party {
    if name == "human" {
        Party::Human
    } else {
        Party::Avatar
    }
}

fn turn_json(turn: &SpokenTurn) -> Value {
    json!({"party": turn.party.to_string(), "text": turn.text})
}

fn add(assembler: &mut TurnAssembler, operation: &Value) -> Vec<SpokenTurn> {
    let party = party_of(text(&operation[1]));
    assembler.add(party, text(&operation[2]), number(&operation[3]))
}

fn apply(assembler: &mut TurnAssembler, operation: &Value) -> Value {
    let turns = match text(&operation[0]) {
        "add" => add(assembler, operation),
        "settled" => assembler.settled(number(&operation[1])),
        _ => assembler.finish(),
    };
    Value::Array(turns.iter().map(turn_json).collect())
}

fn check(case: &Value) {
    let mut assembler = TurnAssembler::new(1.0);
    let produced = array(&case["ops"])
        .iter()
        .map(|operation| apply(&mut assembler, operation));
    same(&Value::Array(produced.collect()), &case["outputs"], case);
}

#[test]
fn every_recorded_python_turn_sequence_is_reproduced() -> Outcome {
    run_cases(FIXTURE, 11, check)
}

fn party_for(human: bool) -> Party {
    if human { Party::Human } else { Party::Avatar }
}

fn feed(additions: &[(bool, String)]) -> String {
    let mut assembler = TurnAssembler::new(1.0);
    let moments = (0i32..).map(|moment| f64::from(moment) * 0.1);
    let mut finished: Vec<SpokenTurn> = Vec::new();
    for ((human, words), moment) in additions.iter().zip(moments) {
        finished.extend(assembler.add(party_for(*human), words, moment));
    }
    finished.extend(assembler.finish());
    finished.iter().map(|turn| turn.text.clone()).collect()
}

proptest! {
    #[test]
    fn every_change_of_speaker_finishes_the_previous_turn_whole(
        additions in prop::collection::vec((any::<bool>(), "[a-z]{1,8}"), 0..20)
    ) {
        let expected: String = additions.iter().map(|(_, words)| words.as_str()).collect();
        prop_assert_eq!(feed(&additions), expected);
    }
}
