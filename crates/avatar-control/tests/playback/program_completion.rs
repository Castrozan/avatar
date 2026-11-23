use std::time::Duration;

use avatar_control::playback::{AudioPlayer, ProgramAudioPlayer};

use super::support::{Scratch, TestResult, clip, killed_by_signal, play_then_stop, within_limit};

#[tokio::test]
async fn stderr_and_exit_code_are_reported_as_error_text() -> TestResult {
    let scratch = Scratch::new("program_stderr")?;
    let program = scratch.fake_program("discord-voice", "printf 'no device\\n' >&2\nexit 7")?;
    let player = ProgramAudioPlayer::new(program, vec!["play".to_owned()]);
    let report = within_limit(player.play(clip()).finished()).await?;
    assert_eq!(report.stderr, "no device");
    assert_eq!(report.exit_status.and_then(|status| status.code()), Some(7));
    Ok(())
}

#[tokio::test]
async fn successful_program_completes_with_zero_status() -> TestResult {
    let scratch = Scratch::new("program_success")?;
    let program = scratch.fake_program("avatar-seat-play", "")?;
    let player = ProgramAudioPlayer::new(program, Vec::new());
    let report = within_limit(player.play(clip()).finished()).await?;
    assert_eq!(report.exit_status.and_then(|status| status.code()), Some(0));
    Ok(())
}

#[tokio::test]
async fn stop_kills_a_running_program() -> TestResult {
    let scratch = Scratch::new("program_stop")?;
    let program = scratch.fake_program("discord-voice", "exec sleep 30")?;
    let player = ProgramAudioPlayer::new(program, vec!["play".to_owned()]);
    let report = play_then_stop(&player).await?;
    assert!(killed_by_signal(&report));
    Ok(())
}

#[tokio::test]
async fn stop_before_program_exits_reports_no_exit_code() -> TestResult {
    let scratch = Scratch::new("program_stop_running")?;
    let program = scratch.fake_program("avatar-seat-play", "exec sleep 30")?;
    let player = ProgramAudioPlayer::new(program, Vec::new());
    let mut playback = player.play(clip());
    tokio::time::sleep(Duration::from_millis(100)).await;
    playback.stop();
    let report = within_limit(playback.finished()).await?;
    assert_eq!(report.exit_status.and_then(|status| status.code()), None);
    Ok(())
}
