use avatar_conversation::talk::ports::Boundaries;
use avatar_conversation::talk::run;
use serde_json::Value;

use super::entries::direct_entry;
use super::fake::World;
use super::{TestResult, fixture_cases};

const FIXTURE: &str = include_str!("fixtures/scenarios.json");

fn run_talk(world: &mut World, scenario: &Value) -> Value {
    let argv: Vec<String> = serde_json::from_value(scenario["argv"].clone()).unwrap_or_default();
    let exit = run(world, &argv);
    world.outcome(exit, None, &Value::Null)
}

fn execute(world: &mut World, scenario: &Value) -> Value {
    let entry = scenario["entry"].as_str().unwrap_or_default();
    if entry == "talk" {
        return run_talk(world, scenario);
    }
    match direct_entry(entry, world, scenario) {
        Ok(value) => world.outcome(0, None, &value),
        Err(failure) => world.outcome(1, Some(failure.message()), &Value::Null),
    }
}

fn check(case: &Value) -> TestResult {
    let mut world = World::new(&case["scenario"]);
    let actual = execute(&mut world, &case["scenario"]);
    if actual != case["expected"] {
        return Err(format!(
            "{}\nexpected {}\nactual   {actual}",
            case["scenario"]["name"], case["expected"]
        )
        .into());
    }
    Ok(())
}

fn assert_boundaries<Candidate: Boundaries>() {}

#[test]
fn the_world_fake_offers_every_boundary() {
    assert_boundaries::<World>();
}

fn failures_of(cases: &[Value]) -> Vec<String> {
    let outcomes = cases.iter().map(check);
    outcomes
        .filter_map(|outcome| outcome.err().map(|error| error.to_string()))
        .collect()
}

#[test]
fn replays_every_scenario_against_the_python_oracle() -> TestResult {
    let cases = fixture_cases(FIXTURE)?;
    let failures = failures_of(&cases);
    assert!(
        failures.is_empty(),
        "{} of {} failed\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n\n")
    );
    Ok(())
}
