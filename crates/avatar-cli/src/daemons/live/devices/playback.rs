use std::io;
use std::process::{Child, Command, Stdio};

use super::sinks::Sink;

const LATENCY_MILLISECONDS: u32 = 80;

fn described(rate: u32, sink: &Sink) -> Vec<String> {
    vec![
        format!("--rate={rate}"),
        format!("--latency-msec={LATENCY_MILLISECONDS}"),
        format!("--client-name={}", sink.client),
    ]
}

fn pacat_arguments(rate: u32, sink: &Sink) -> Vec<String> {
    let fixed = ["--playback", "--raw", "--channels=1", "--format=s16le"];
    let device = sink.device.map(|name| format!("--device={name}"));
    let leading = fixed.map(str::to_owned).into_iter();
    leading.chain(described(rate, sink)).chain(device).collect()
}

pub(super) fn start_playback(rate: u32, sink: &Sink) -> io::Result<Child> {
    Command::new("pacat")
        .args(pacat_arguments(rate, sink))
        .stdin(Stdio::piped())
        .spawn()
}
