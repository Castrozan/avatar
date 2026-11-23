use avatar_control::speaking::runtime::{Platform, PlaybackTarget, playback_targets};
use avatar_control::speaking::{OutputRoute, output_plan};
use proptest::prelude::*;
use serde_json::Value;

fn output_name() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("speakers".to_owned()),
        Just("mic".to_owned()),
        Just("both".to_owned()),
        Just("discord".to_owned()),
        "\\PC*",
    ]
}

fn route_of(name: &str) -> OutputRoute {
    output_plan(&Value::from(name)).route
}

proptest! {
    #[test]
    fn linux_plays_one_target_per_planned_sink(name in output_name()) {
        let route = route_of(&name);
        let targets = playback_targets(&route, Platform::Linux);
        let expected = match &route {
            OutputRoute::Sinks(sinks) => sinks.len(),
            OutputRoute::Discord => 1,
        };
        prop_assert_eq!(targets.len(), expected);
    }

    #[test]
    fn discord_is_never_rerouted_by_platform(name in output_name()) {
        let route = route_of(&name);
        for platform in [Platform::Linux, Platform::Darwin] {
            let is_discord = playback_targets(&route, platform) == vec![PlaybackTarget::Discord];
            prop_assert_eq!(is_discord, name == "discord");
        }
    }

    #[test]
    fn darwin_plays_the_seat_for_every_non_discord_output(name in output_name()) {
        let targets = playback_targets(&route_of(&name), Platform::Darwin);
        let expected = if name == "discord" { PlaybackTarget::Discord } else { PlaybackTarget::Seat };
        prop_assert_eq!(targets, vec![expected]);
    }
}
