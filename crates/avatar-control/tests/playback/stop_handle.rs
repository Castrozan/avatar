use std::time::Duration;

use avatar_control::playback::{AudioPlayer, ProgramAudioPlayer};

use super::support::{Scratch, TestResult, clip, killed_by_signal, within_limit};

#[tokio::test]
async fn stop_handle_kills_a_program_while_another_task_awaits_finish() -> TestResult {
    let scratch = Scratch::new("stop_handle_cross_task")?;
    let program = scratch.fake_program("paplay", "exec sleep 30")?;
    let playback = ProgramAudioPlayer::new(program, Vec::new()).play(clip());
    let stopper = playback.stop_handle();
    let waiter = tokio::spawn(playback.finished());
    tokio::time::sleep(Duration::from_millis(50)).await;
    stopper.stop();
    let report = within_limit(waiter).await??;
    assert!(killed_by_signal(&report));
    Ok(())
}

#[tokio::test]
async fn stop_handle_clones_share_one_stop_request() -> TestResult {
    let scratch = Scratch::new("stop_handle_clones")?;
    let program = scratch.fake_program("paplay", "exec sleep 30")?;
    let playback = ProgramAudioPlayer::new(program, Vec::new()).play(clip());
    let first = playback.stop_handle();
    let second = first.clone();
    first.stop();
    second.stop();
    let report = within_limit(playback.finished()).await?;
    assert!(killed_by_signal(&report));
    Ok(())
}
