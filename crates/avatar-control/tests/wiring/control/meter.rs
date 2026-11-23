use std::fs;

use avatar_control::control::MicrophoneLevelMeter;
use tokio::sync::mpsc::unbounded_channel;
use tokio::time::timeout;

use crate::rigs::meter::{
    ChannelSink, EXPECTED_ARGUMENTS, WAIT, last_level, playing_recorder, settings, wait_for_level,
    wait_for_starts,
};
use crate::support::{TestResult, check, fake_executable};

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
