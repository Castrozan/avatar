use std::path::Path;
use std::time::Duration;

use avatar_control::playback::{AudioPlayer, PulseSinkPlayer};

use super::support::{Scratch, TestResult, clip, killed_by_signal, play_then_stop, within_limit};

#[tokio::test]
async fn stderr_and_exit_code_are_reported_as_error_text() -> TestResult {
    let scratch = Scratch::new("pulse_stderr")?;
    let program = scratch.fake_program("paplay", "printf 'boom\\n' >&2\nexit 3")?;
    let player = PulseSinkPlayer::new(program, "AvatarMic", "/run/user/1000");
    let report = within_limit(player.play(clip()).finished()).await?;
    assert_eq!(report.stderr, "boom");
    assert_eq!(report.exit_status.and_then(|status| status.code()), Some(3));
    Ok(())
}

#[tokio::test]
async fn successful_program_completes_with_zero_status() -> TestResult {
    let scratch = Scratch::new("pulse_success")?;
    let program = scratch.fake_program("paplay", "")?;
    let player = PulseSinkPlayer::new(program, "AvatarMic", "/run/user/1000");
    let report = within_limit(player.play(clip()).finished()).await?;
    assert_eq!(report.exit_status.and_then(|status| status.code()), Some(0));
    assert!(report.stderr.is_empty());
    Ok(())
}

#[tokio::test]
async fn stop_kills_a_running_program() -> TestResult {
    let scratch = Scratch::new("pulse_stop")?;
    let program = scratch.fake_program("paplay", "exec sleep 30")?;
    let player = PulseSinkPlayer::new(program, "AvatarMic", "/run/user/1000");
    let report = play_then_stop(&player).await?;
    assert!(killed_by_signal(&report));
    Ok(())
}

#[tokio::test]
async fn stop_twice_is_harmless() -> TestResult {
    let scratch = Scratch::new("pulse_stop_twice")?;
    let program = scratch.fake_program("paplay", "exec sleep 30")?;
    let mut playback = PulseSinkPlayer::new(program, "AvatarMic", "/run/user/1000").play(clip());
    playback.stop();
    playback.stop();
    let report = within_limit(playback.finished()).await?;
    assert!(killed_by_signal(&report));
    Ok(())
}

#[tokio::test]
async fn dropping_the_handle_does_not_stop_playback() -> TestResult {
    let scratch = Scratch::new("pulse_drop")?;
    let marker = scratch.marker_path();
    let body = format!("sleep 0.2\nprintf done > {}", marker.display());
    let program = scratch.fake_program("paplay", &body)?;
    let player = PulseSinkPlayer::new(program, "AvatarMic", "/run/user/1000");
    drop(player.play(clip()));
    wait_for_marker(&marker).await
}

async fn wait_for_marker(marker: &Path) -> TestResult {
    within_limit(wait_until_present(marker)).await
}

async fn wait_until_present(marker: &Path) {
    while !marker.exists() {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
