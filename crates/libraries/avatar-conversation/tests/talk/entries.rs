use avatar_conversation::talk::state::seat::{parse_seat, seat_microphone_ready};
use avatar_conversation::talk::surfaces::desktop;
use avatar_conversation::talk::surfaces::discord;
use avatar_conversation::talk::surfaces::seat_preparation::prepare;
use avatar_conversation::talk::surfaces::window::reveal_window;
use avatar_conversation::talk::{TalkFailure, speech::speak_on_seat};
use serde_json::{Value, json};

use super::fake::World;

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

pub(super) fn direct_entry(entry: &str, world: &mut World, scenario: &Value) -> Entry {
    match entry {
        "reveal_window" => reveal_window(world).map(|()| Value::Null),
        "desktop_show" => desktop::show(world).map(|()| Value::Null),
        "desktop_hide" => desktop::hide(world).map(|()| Value::Null),
        entry if entry.starts_with("discord") => discord_entry(entry, world),
        entry => seat_entry(entry, world, scenario),
    }
}
