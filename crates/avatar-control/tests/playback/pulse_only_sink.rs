use std::io;

use avatar_control::playback::{AudioPlayer, PulseSinkPlayer};

use super::support::{CLIP_PATH, Scratch, TestResult, clip, within_limit};

fn expected_lip_sync_arguments() -> Vec<String> {
    [
        "--client-name",
        "avatar-speech",
        "--device",
        "AvatarLipSync",
    ]
    .into_iter()
    .chain(["--property=node.dont-fallback=true", CLIP_PATH])
    .map(str::to_owned)
    .collect()
}

fn lip_sync_player(scratch: &Scratch) -> io::Result<PulseSinkPlayer> {
    let program = scratch.fake_program("paplay", "")?;
    Ok(PulseSinkPlayer::only_sink(
        program,
        "AvatarLipSync",
        "/run/user/1000",
    ))
}

#[tokio::test]
async fn lip_sync_sink_argv_matches_js_builder() -> TestResult {
    let scratch = Scratch::new("pulse_only_lip_sync_argv")?;
    let player = lip_sync_player(&scratch)?;
    within_limit(player.play(clip()).finished()).await?;
    assert_eq!(scratch.recorded_arguments()?, expected_lip_sync_arguments());
    Ok(())
}
