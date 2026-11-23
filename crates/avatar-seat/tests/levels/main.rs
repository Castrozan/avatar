use std::collections::BTreeMap;

use avatar_seat::host::levels::{ChannelVolume, Sink, levels_from_sinks, sink_gain};
use proptest::prelude::*;
use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Deserialize)]
struct Case {
    sinks: Vec<Sink>,
    expected: Map<String, Value>,
}

fn level_bits(level: &Value) -> u64 {
    level.as_f64().unwrap_or(f64::NAN).to_bits()
}

fn bits(map: &Map<String, Value>) -> Vec<(String, u64)> {
    map.iter()
        .map(|(channel, level)| (channel.clone(), level_bits(level)))
        .collect()
}

fn check_case(case: &Case) {
    assert_eq!(bits(&levels_from_sinks(&case.sinks)), bits(&case.expected));
}

#[test]
fn levels_match_the_python_oracle() {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/levels.json")).unwrap_or_default();
    assert!(cases.len() >= 10);
    cases.iter().for_each(check_case);
}

fn channel(index: usize, percent: f64) -> (String, ChannelVolume) {
    (
        index.to_string(),
        ChannelVolume {
            value_percent: format!("{percent}%"),
        },
    )
}

fn sink_with(percents: &[f64], mute: bool) -> Sink {
    let volume: BTreeMap<String, ChannelVolume> = percents
        .iter()
        .enumerate()
        .map(|(index, percent)| channel(index, *percent))
        .collect();
    Sink {
        name: "AvatarMic".to_owned(),
        mute,
        volume,
    }
}

fn gain_is_bounded(percents: &[f64], mute: bool) -> bool {
    let gain = sink_gain(&sink_with(percents, mute));
    (0.0..=1.0).contains(&gain) && (!mute || gain == 0.0)
}

proptest! {
    #[test]
    fn a_gain_always_lies_between_silence_and_full(percents in prop::collection::vec(-500.0..500.0f64, 0..6), mute in any::<bool>()) {
        prop_assert!(gain_is_bounded(&percents, mute));
    }
}
