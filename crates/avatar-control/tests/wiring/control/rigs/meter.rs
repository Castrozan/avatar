use std::fs;
use std::path::Path;
use std::time::Duration;

use avatar_control::control::{LevelSink, MeterSettings};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::timeout;

use crate::support::{TestResult, fake_executable};

pub(crate) const WAIT: Duration = Duration::from_secs(10);
pub(crate) const EXPECTED_ARGUMENTS: &str = "--device\nroom.monitor\n--property=node.dont-reconnect=true\n--raw\n--format=float32le\n--rate=48000\n--channels=1\n--latency-msec=20\n";

#[derive(Clone)]
pub(crate) struct ChannelSink(pub(crate) UnboundedSender<f64>);

impl LevelSink for ChannelSink {
    async fn send_level(&self, level: f64) {
        let _ = self.0.send(level);
    }
}

pub(crate) fn settings(program: &Path) -> MeterSettings {
    MeterSettings {
        program: program.to_string_lossy().into_owned(),
        source: "room.monitor".to_owned(),
        listen_again: Duration::from_millis(20),
        report_every: Duration::from_millis(10),
    }
}

pub(crate) async fn next_level(
    levels: &mut UnboundedReceiver<f64>,
) -> Result<f64, Box<dyn std::error::Error>> {
    Ok(timeout(WAIT, levels.recv()).await?.ok_or("sink closed")?)
}

pub(crate) async fn wait_for_level(
    levels: &mut UnboundedReceiver<f64>,
    expected: f64,
) -> TestResult {
    while next_level(levels).await?.to_bits() != expected.to_bits() {}
    Ok(())
}

pub(crate) async fn wait_for_starts(
    counter: &Path,
    levels: &mut UnboundedReceiver<f64>,
) -> TestResult {
    while start_count(counter) < 3 {
        next_level(levels).await?;
    }
    Ok(())
}

pub(crate) fn half_scale_samples() -> Vec<u8> {
    (0..480).flat_map(|_| 0.5_f32.to_le_bytes()).collect()
}

pub(crate) fn playing_recorder(
    directory: &Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let samples = directory.join("samples.bin");
    fs::write(&samples, half_scale_samples())?;
    let record = format!(
        "printf '%s\\n' \"$@\" > {}",
        directory.join("args").display()
    );
    let script = format!("{record}\ncat {}\nexec sleep 30", samples.display());
    fake_executable(directory, "parec", &script)
}

pub(crate) fn last_level(levels: &mut UnboundedReceiver<f64>) -> f64 {
    let mut last = f64::NAN;
    while let Ok(level) = levels.try_recv() {
        last = level;
    }
    last
}

pub(crate) fn start_count(counter: &Path) -> usize {
    fs::read_to_string(counter).map_or(0, |text| text.lines().count())
}
