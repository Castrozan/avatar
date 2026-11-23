use std::collections::BTreeMap;
use std::time::Duration;

use super::harness::{Outcome, Running, Socket};
use avatar_seat::host::levels::{ChannelVolume, Sink};
use futures_util::SinkExt;
use serde_json::{Value, json};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

pub(crate) fn volume(percent: u32) -> BTreeMap<String, ChannelVolume> {
    let channel = ChannelVolume {
        value_percent: format!("{percent}%"),
    };
    BTreeMap::from([("mono".to_owned(), channel)])
}

pub(crate) fn sink(name: &str, percent: u32, mute: bool) -> Sink {
    Sink {
        name: name.to_owned(),
        mute,
        volume: volume(percent),
    }
}

pub(crate) fn quiet_levels(running: &Running) {
    set_sinks(
        running,
        vec![sink("AvatarMic", 50, false), sink("AvatarHuman", 100, true)],
    );
}

pub(crate) fn quiet_answer() -> Value {
    json!({"agent": 0.5, "human": 0.0})
}

pub(crate) fn loud_answer() -> Value {
    json!({"agent": 0.2, "human": 0.8})
}

pub(crate) fn loud_levels(running: &Running) {
    set_sinks(
        running,
        vec![sink("AvatarMic", 20, false), sink("AvatarHuman", 80, false)],
    );
}

pub(crate) fn set_sinks(running: &Running, sinks: Vec<Sink>) {
    if let Ok(mut held) = running.levels.sinks.lock() {
        *held = sinks;
    }
}

pub(crate) fn reads(running: &Running) -> usize {
    running
        .levels
        .reads
        .load(std::sync::atomic::Ordering::SeqCst)
}

pub(crate) async fn attach(running: &Running) -> Outcome<Socket> {
    let mut socket = running.socket("/audio", None).await?;
    socket
        .send(Message::Binary(MICROPHONE.to_vec().into()))
        .await?;
    Ok(socket)
}

pub(crate) async fn poll_released(running: &Running) -> Outcome<Value> {
    let mut health = running.health().await?;
    while health["attached"] != false {
        tokio::time::sleep(Duration::from_millis(10)).await;
        health = running.health().await?;
    }
    Ok(health)
}

pub(crate) async fn wait_released(running: &Running) -> Outcome<Value> {
    timeout(FIVE_SECONDS, poll_released(running)).await?
}

async fn poll_meeting(running: &Running) -> Outcome<Value> {
    let mut health = running.health().await?;
    while health["meeting"] != true {
        tokio::time::sleep(Duration::from_millis(10)).await;
        health = running.health().await?;
    }
    Ok(health)
}

pub(crate) async fn wait_meeting(running: &Running) -> Outcome<Value> {
    timeout(FIVE_SECONDS, poll_meeting(running)).await?
}

pub(crate) fn flags(health: &Value) -> (Value, Value) {
    (health["attached"].clone(), health["meeting"].clone())
}

pub(crate) const MICROPHONE: [u8; 640] = [3; 640];
pub(crate) const LONG: (u64, u64) = (30_000, 10_000);
pub(crate) const FIVE_SECONDS: Duration = Duration::from_secs(5);
