use std::path::Path;

use avatar_control::speaking::runtime::{PlaybackTarget, PlayerPrograms, Players, SinkPlayers};

use super::executables::write_executable;
use crate::TestResult;

const RUNTIME_DIRECTORY: &str = "/run/user/4242";
const CLIP: &str = "/tmp/fixture/clip.wav";
const ONLY_SINK: &str = "--property=node.dont-fallback=true ";

struct Recorder {
    name: &'static str,
    pattern: &'static str,
    arguments: &'static str,
}

const PAPLAY: Recorder = Recorder {
    name: "paplay",
    pattern: "paplay|%s|%s",
    arguments: r#""$XDG_RUNTIME_DIR" "$*""#,
};
const DISCORD: Recorder = Recorder {
    name: "discord-voice",
    pattern: "discord-voice|%s",
    arguments: r#""$*""#,
};
const SEAT: Recorder = Recorder {
    name: "avatar-seat-play",
    pattern: "avatar-seat-play|%s",
    arguments: r#""$*""#,
};

fn install(directory: &Path, recorder: &Recorder) -> Result<String, std::io::Error> {
    let log = directory.join("plays.log");
    let path = directory.join(recorder.name);
    let (pattern, arguments) = (recorder.pattern, recorder.arguments);
    write_executable(
        &path,
        &format!("printf '{pattern}\\n' {arguments} >> {}", log.display()),
    )?;
    Ok(path.to_string_lossy().into_owned())
}

fn programs(directory: &Path) -> Result<PlayerPrograms, std::io::Error> {
    Ok(PlayerPrograms {
        paplay: install(directory, &PAPLAY)?,
        discord_voice: install(directory, &DISCORD)?,
        seat_play: install(directory, &SEAT)?,
    })
}

const TARGETS: [PlaybackTarget; 5] = [
    PlaybackTarget::LipSync,
    PlaybackTarget::Mic,
    PlaybackTarget::Speakers,
    PlaybackTarget::Discord,
    PlaybackTarget::Seat,
];

async fn play_all(players: &SinkPlayers) {
    for target in TARGETS {
        players.start(target, Path::new(CLIP)).finished.await;
    }
}

fn pulse_line(device: &str, extra: &str) -> String {
    let arguments = format!("--client-name avatar-speech --device {device} {extra}{CLIP}");
    format!("paplay|{RUNTIME_DIRECTORY}|{arguments}")
}

fn expected_lines() -> Vec<String> {
    vec![
        pulse_line("AvatarLipSync", ONLY_SINK),
        pulse_line("AvatarMic", ONLY_SINK),
        pulse_line("default", ""),
        format!("discord-voice|play {CLIP}"),
        format!("avatar-seat-play|{CLIP}"),
    ]
}

#[tokio::test]
async fn every_target_runs_its_own_program_with_its_own_arguments() -> TestResult {
    let directory = tempfile::tempdir()?;
    let players = SinkPlayers::new(&programs(directory.path())?, RUNTIME_DIRECTORY);
    play_all(&players).await;
    let log = std::fs::read_to_string(directory.path().join("plays.log"))?;
    assert_eq!(log.lines().collect::<Vec<_>>(), expected_lines());
    Ok(())
}
