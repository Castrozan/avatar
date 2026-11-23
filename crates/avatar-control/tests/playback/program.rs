use avatar_control::playback::{AudioPlayer, ProgramAudioPlayer};

use super::support::{CLIP_PATH, Scratch, TestResult, clip, has_environment_line, within_limit};

#[tokio::test]
async fn program_argv_matches_js_builder_for_discord_voice() -> TestResult {
    let scratch = Scratch::new("program_discord_argv")?;
    let program = scratch.fake_program("discord-voice", "")?;
    let player = ProgramAudioPlayer::new(program, vec!["play".to_owned()]);
    within_limit(player.play(clip()).finished()).await?;
    assert_eq!(scratch.recorded_arguments()?, ["play", CLIP_PATH]);
    Ok(())
}

#[tokio::test]
async fn audio_path_is_the_only_argument_without_configured_arguments() -> TestResult {
    let scratch = Scratch::new("program_path_only")?;
    let program = scratch.fake_program("avatar-seat-play", "")?;
    let player = ProgramAudioPlayer::new(program, Vec::new());
    within_limit(player.play(clip()).finished()).await?;
    assert_eq!(scratch.recorded_arguments()?, [CLIP_PATH]);
    Ok(())
}

#[tokio::test]
async fn stdin_is_closed_and_empty() -> TestResult {
    let scratch = Scratch::new("program_stdin")?;
    let program = scratch.fake_program("avatar-seat-play", "")?;
    let player = ProgramAudioPlayer::new(program, Vec::new());
    within_limit(player.play(clip()).finished()).await?;
    assert!(scratch.recorded_stdin()?.is_empty());
    Ok(())
}

#[tokio::test]
async fn parent_environment_is_inherited_unchanged() -> TestResult {
    let scratch = Scratch::new("program_env")?;
    let program = scratch.fake_program("avatar-seat-play", "")?;
    let path = std::env::var("PATH").unwrap_or_default();
    let player = ProgramAudioPlayer::new(program, Vec::new());
    within_limit(player.play(clip()).finished()).await?;
    assert!(has_environment_line(
        &scratch.recorded_environment()?,
        &format!("PATH={path}")
    ));
    Ok(())
}

#[tokio::test]
async fn missing_program_completes_without_status_like_the_js_error_event() -> TestResult {
    let player = ProgramAudioPlayer::new("/nonexistent/discord-voice", vec!["play".to_owned()]);
    let report = within_limit(player.play(clip()).finished()).await?;
    assert!(report.exit_status.is_none());
    assert!(report.stderr.is_empty());
    Ok(())
}

#[tokio::test]
async fn playback_is_usable_through_the_trait_object() -> TestResult {
    let scratch = Scratch::new("program_trait_object")?;
    let program = scratch.fake_program("avatar-seat-play", "")?;
    let player: Box<dyn AudioPlayer> = Box::new(ProgramAudioPlayer::new(program, Vec::new()));
    let report = within_limit(player.play(clip()).finished()).await?;
    assert_eq!(scratch.recorded_arguments()?, [CLIP_PATH]);
    assert_eq!(report.exit_status.and_then(|status| status.code()), Some(0));
    Ok(())
}
