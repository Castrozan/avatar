use avatar_conversation::talk::ports::Boundaries;
use avatar_conversation::talk::state::seat::{parse_seat, seat_microphone_ready};
use avatar_conversation::talk::surfaces::desktop;
use avatar_conversation::talk::surfaces::discord;
use avatar_conversation::talk::surfaces::seat_preparation::prepare;
use avatar_conversation::talk::surfaces::window::reveal_window;
use avatar_conversation::talk::{TalkFailure, run, speech::speak_on_seat};
use serde_json::{Value, json};

use super::fake::World;
use super::{TestResult, fixture_cases};

const FIXTURE: &str = include_str!("fixtures/scenarios.json");

type Entry = Result<Value, TalkFailure>;

fn seat_argument(scenario: &Value) -> Result<avatar_floor::quiet::Seat, TalkFailure> {
    let name = scenario["arg"].as_str().unwrap_or_default();
    parse_seat(name).ok_or_else(|| TalkFailure::new(format!("unknown seat {name}")))
}

fn sources(scenario: &Value) -> Vec<String> {
    let listed = scenario["existing_sources"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    listed
        .iter()
        .filter_map(|source| source.as_str().map(str::to_owned))
        .collect()
}

fn discord_entry(entry: &str, world: &mut World) -> Entry {
    match entry {
        "discord_join" => discord::join(world).map(|()| Value::Null),
        "discord_in_call" => Ok(json!(discord::in_call(world))),
        "discord_listener_left" => Ok(json!(discord::listener_left_the_call(world))),
        _ => Ok(Value::Null).inspect(|_| discord::leave(world)),
    }
}

fn seat_entry(entry: &str, world: &mut World, scenario: &Value) -> Entry {
    let seat = seat_argument(scenario)?;
    match entry {
        "seat_prepare" => prepare(world, seat).map(|()| Value::Null),
        "speak_on_seat" => {
            speak_on_seat(world, scenario["text"].as_str().unwrap_or_default(), seat)
                .map(|()| Value::Null)
        }
        _ => Ok(json!(seat_microphone_ready(seat, &sources(scenario)))),
    }
}

fn direct_entry(entry: &str, world: &mut World, scenario: &Value) -> Entry {
    match entry {
        "reveal_window" => reveal_window(world).map(|()| Value::Null),
        "desktop_show" => desktop::show(world).map(|()| Value::Null),
        "desktop_hide" => desktop::hide(world).map(|()| Value::Null),
        entry if entry.starts_with("discord") => discord_entry(entry, world),
        entry => seat_entry(entry, world, scenario),
    }
}

fn execute(world: &mut World, scenario: &Value) -> Value {
    let entry = scenario["entry"].as_str().unwrap_or_default();
    if entry == "talk" {
        let argv: Vec<String> =
            serde_json::from_value(scenario["argv"].clone()).unwrap_or_default();
        let exit = run(world, &argv);
        return world.outcome(exit, None, &Value::Null);
    }
    match direct_entry(entry, world, scenario) {
        Ok(value) => world.outcome(0, None, &value),
        Err(failure) => world.outcome(1, Some(failure.message().to_owned()), &Value::Null),
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

#[test]
fn replays_every_scenario_against_the_python_oracle() -> TestResult {
    let cases = fixture_cases(FIXTURE)?;
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|case| check(case).err().map(|error| error.to_string()))
        .collect();
    assert!(
        failures.is_empty(),
        "{} of {} failed\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n\n")
    );
    Ok(())
}
