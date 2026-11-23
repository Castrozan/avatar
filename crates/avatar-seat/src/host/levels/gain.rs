use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{Map, Value};

use super::CHANNELS;

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelVolume {
    pub value_percent: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Sink {
    pub name: String,
    pub mute: bool,
    pub volume: BTreeMap<String, ChannelVolume>,
}

fn percent_of(channel: &ChannelVolume) -> Option<f64> {
    channel
        .value_percent
        .trim_end_matches('%')
        .trim()
        .parse()
        .ok()
}

fn mean_percent(sink: &Sink) -> f64 {
    let percents: Vec<f64> = sink.volume.values().filter_map(percent_of).collect();
    let count = f64::from(u32::try_from(percents.len()).unwrap_or(u32::MAX));
    percents.iter().sum::<f64>() / count
}

#[must_use]
pub fn sink_gain(sink: &Sink) -> f64 {
    let mean = mean_percent(sink);
    if sink.mute || mean.is_nan() {
        return 0.0;
    }
    (mean / 100.0).clamp(0.0, 1.0)
}

fn channel_level(sinks: &[Sink], channel: &str, name: &str) -> Option<(String, Value)> {
    let sink = sinks.iter().rfind(|candidate| candidate.name == name)?;
    Some((channel.to_owned(), Value::from(sink_gain(sink))))
}

#[must_use]
pub fn levels_from_sinks(sinks: &[Sink]) -> Map<String, Value> {
    let levels = CHANNELS
        .iter()
        .map(|(channel, name)| channel_level(sinks, channel, name));
    levels.flatten().collect()
}
