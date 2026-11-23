use std::fs;
use std::path::Path;
use std::time::Duration;

use avatar_control::control::{LevelSink, MeterSettings, MicrophoneLevelMeter};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::time::timeout;

use crate::support::{TestResult, check, fake_executable};

const WAIT: Duration = Duration::from_secs(10);
const EXPECTED_ARGUMENTS: &str = "--device\nroom.monitor\n--property=node.dont-reconnect=true\n--raw\n--format=float32le\n--rate=48000\n--channels=1\n--latency-msec=20\n";

#[derive(Clone)]
struct ChannelSink(UnboundedSender<f64>);

impl LevelSink for ChannelSink {
    async fn send_level(&self, level: f64) {
        let _ = self.0.send(level);
    }
}

fn settings(program: &Path) -> MeterSettings {
    MeterSettings {
        program: program.to_string_lossy().into_owned(),
        source: "room.monitor".to_owned(),
        listen_again: Duration::from_millis(20),
        report_every: Duration::from_millis(10),
    }
}

async fn next_level(
    levels: &mut UnboundedReceiver<f64>,
) -> Result<f64, Box<dyn std::error::Error>> {
    Ok(timeout(WAIT, levels.recv()).await?.ok_or("sink closed")?)
}

async fn wait_for_level(levels: &mut UnboundedReceiver<f64>, expected: f64) -> TestResult {
    while next_level(levels).await?.to_bits() != expected.to_bits() {}
    Ok(())
}

async fn wait_for_starts(counter: &Path, levels: &mut UnboundedReceiver<f64>) -> TestResult {
    while start_count(counter) < 3 {
        next_level(levels).await?;
    }
    Ok(())
}

fn half_scale_samples() -> Vec<u8> {
    (0..480).flat_map(|_| 0.5_f32.to_le_bytes()).collect()
}

fn playing_recorder(directory: &Path) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let samples = directory.join("samples.bin");
    fs::write(&samples, half_scale_samples())?;
    let record = format!(
        "printf '%s\\n' \"$@\" > {}",
        directory.join("args").display()
    );
    let script = format!("{record}\ncat {}\nexec sleep 30", samples.display());
    fake_executable(directory, "parec", &script)
}

#[tokio::test]
async fn recorder_is_started_with_the_node_arguments_and_levels_are_reported() -> TestResult {
    let directory = tempfile::tempdir()?;
    let program = playing_recorder(directory.path())?;
    let (sink, mut levels) = unbounded_channel();
    let meter = MicrophoneLevelMeter::start(settings(&program), ChannelSink(sink));
    wait_for_level(&mut levels, 0.5).await?;
    meter.stop().await;
    let arguments = fs::read_to_string(directory.path().join("args"))?;
    check(arguments == EXPECTED_ARGUMENTS, arguments)
}

fn last_level(levels: &mut UnboundedReceiver<f64>) -> f64 {
    let mut last = f64::NAN;
    while let Ok(level) = levels.try_recv() {
        last = level;
    }
    last
}

#[tokio::test]
async fn a_final_silence_is_reported_when_the_meter_stops() -> TestResult {
    let directory = tempfile::tempdir()?;
    let program = fake_executable(directory.path(), "parec", "exec sleep 30")?;
    let (sink, mut levels) = unbounded_channel();
    let meter = MicrophoneLevelMeter::start(settings(&program), ChannelSink(sink));
    meter.stop().await;
    let last = last_level(&mut levels);
    check(
        last.to_bits() == 0.0_f64.to_bits(),
        format!("last level {last}"),
    )
}

fn start_count(counter: &Path) -> usize {
    fs::read_to_string(counter).map_or(0, |text| text.lines().count())
}

#[tokio::test]
async fn the_recorder_is_started_again_after_it_exits() -> TestResult {
    let directory = tempfile::tempdir()?;
    let counter = directory.path().join("starts");
    let script = format!("echo started >> {}", counter.display());
    let program = fake_executable(directory.path(), "parec", &script)?;
    let (sink, mut levels) = unbounded_channel();
    let meter = MicrophoneLevelMeter::start(settings(&program), ChannelSink(sink));
    wait_for_starts(&counter, &mut levels).await?;
    meter.stop().await;
    Ok(())
}

#[tokio::test]
async fn a_missing_recorder_program_keeps_trying_and_stops_cleanly() -> TestResult {
    let directory = tempfile::tempdir()?;
    let (sink, _levels) = unbounded_channel();
    let missing = directory.path().join("absent");
    let meter = MicrophoneLevelMeter::start(settings(&missing), ChannelSink(sink));
    Ok(timeout(WAIT, meter.stop()).await?)
}
