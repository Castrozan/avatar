use avatar_control::playback::{AudioPlayer, PulseSinkPlayer};

use super::support::{CLIP_PATH, Scratch, TestResult, clip, has_environment_line, within_limit};

fn expected_pulse_arguments(sink: &str) -> Vec<String> {
    vec![
        "--client-name",
        "avatar-speech",
        "--device",
        sink,
        CLIP_PATH,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

#[tokio::test]
async fn shared_sink_argv_has_no_fallback_property() -> TestResult {
    let scratch = Scratch::new("pulse_shared_argv")?;
    let program = scratch.fake_program("paplay", "")?;
    let player = PulseSinkPlayer::new(program, "AvatarLipSync", "/run/user/1000");
    within_limit(player.play(clip()).finished()).await?;
    assert_eq!(
        scratch.recorded_arguments()?,
        expected_pulse_arguments("AvatarLipSync")
    );
    Ok(())
}

#[tokio::test]
async fn default_sink_argv_matches_js_builder() -> TestResult {
    let scratch = Scratch::new("pulse_default_argv")?;
    let program = scratch.fake_program("paplay", "")?;
    let player = PulseSinkPlayer::new(program, "default", "/run/user/1000");
    within_limit(player.play(clip()).finished()).await?;
    assert_eq!(
        scratch.recorded_arguments()?,
        expected_pulse_arguments("default")
    );
    Ok(())
}

#[tokio::test]
async fn runtime_directory_is_exported_as_xdg_runtime_dir() -> TestResult {
    let scratch = Scratch::new("pulse_xdg_runtime")?;
    let program = scratch.fake_program("paplay", "")?;
    let player = PulseSinkPlayer::new(program, "AvatarMic", "/run/user/1000");
    within_limit(player.play(clip()).finished()).await?;
    let environment = scratch.recorded_environment()?;
    assert!(has_environment_line(
        &environment,
        "XDG_RUNTIME_DIR=/run/user/1000"
    ));
    Ok(())
}

#[tokio::test]
async fn runtime_directory_follows_the_configured_user() -> TestResult {
    let scratch = Scratch::new("pulse_xdg_other_user")?;
    let program = scratch.fake_program("paplay", "")?;
    let player = PulseSinkPlayer::new(program, "AvatarMic", "/run/user/4242");
    within_limit(player.play(clip()).finished()).await?;
    let environment = scratch.recorded_environment()?;
    assert!(has_environment_line(
        &environment,
        "XDG_RUNTIME_DIR=/run/user/4242"
    ));
    Ok(())
}

#[tokio::test]
async fn parent_environment_is_inherited() -> TestResult {
    let scratch = Scratch::new("pulse_inherited_env")?;
    let program = scratch.fake_program("paplay", "")?;
    let path = std::env::var("PATH").unwrap_or_default();
    let player = PulseSinkPlayer::new(program, "AvatarMic", "/run/user/1000");
    within_limit(player.play(clip()).finished()).await?;
    let environment = scratch.recorded_environment()?;
    assert!(has_environment_line(&environment, &format!("PATH={path}")));
    Ok(())
}

#[tokio::test]
async fn stdin_is_closed_and_empty() -> TestResult {
    let scratch = Scratch::new("pulse_stdin")?;
    let program = scratch.fake_program("paplay", "")?;
    let player = PulseSinkPlayer::new(program, "AvatarMic", "/run/user/1000");
    within_limit(player.play(clip()).finished()).await?;
    assert!(scratch.recorded_stdin()?.is_empty());
    Ok(())
}

#[tokio::test]
async fn missing_program_completes_without_status_like_the_js_error_event() -> TestResult {
    let player = PulseSinkPlayer::new("/nonexistent/avatar-paplay", "AvatarMic", "/run/user/1000");
    let report = within_limit(player.play(clip()).finished()).await?;
    assert!(report.exit_status.is_none());
    assert!(report.stderr.is_empty());
    Ok(())
}
