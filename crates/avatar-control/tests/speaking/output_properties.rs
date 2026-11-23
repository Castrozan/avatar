use avatar_control::speaking::{OutputRoute, SinkRole, output_plan};
use proptest::prelude::*;
use serde_json::Value;

fn sinks_of(route: &OutputRoute) -> Option<&Vec<SinkRole>> {
    match route {
        OutputRoute::Sinks(sinks) => Some(sinks),
        OutputRoute::Discord => None,
    }
}

fn output_name() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("speakers".to_owned()),
        Just("mic".to_owned()),
        Just("both".to_owned()),
        Just("discord".to_owned()),
        "\\PC*",
    ]
}

proptest! {
    #[test]
    fn agent_channel_is_used_exactly_when_the_mic_is_fed(name in output_name()) {
        let plan = output_plan(&Value::from(name.as_str()));
        let feeds_mic = sinks_of(&plan.route).is_some_and(|sinks| sinks.contains(&SinkRole::Mic));
        prop_assert_eq!(plan.uses_agent_channel, feeds_mic);
    }

    #[test]
    fn lip_sync_leads_every_sink_route(name in output_name()) {
        let plan = output_plan(&Value::from(name.as_str()));
        if let Some(sinks) = sinks_of(&plan.route) {
            prop_assert_eq!(sinks.first(), Some(&SinkRole::LipSync));
        }
    }

    #[test]
    fn speakers_are_fed_only_for_speakers_or_both(name in output_name()) {
        let plan = output_plan(&Value::from(name.as_str()));
        let expected = matches!(name.as_str(), "speakers" | "both");
        let feeds_speakers = sinks_of(&plan.route).is_some_and(|sinks| sinks.contains(&SinkRole::Speakers));
        prop_assert_eq!(feeds_speakers, expected);
    }

    #[test]
    fn discord_route_is_exactly_the_discord_name(name in output_name()) {
        let plan = output_plan(&Value::from(name.as_str()));
        prop_assert_eq!(plan.route == OutputRoute::Discord, name == "discord");
    }
}
