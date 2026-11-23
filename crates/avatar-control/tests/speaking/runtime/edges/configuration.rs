use avatar_control::speaking::runtime::{Platform, SpeechRuntimeConfiguration, agent_channel_for};

use crate::TestResult;

const MISSING_PROGRAM: &str = "/nonexistent/avatar-audio";

#[tokio::test]
async fn darwin_uses_the_seat_channel_without_running_avatar_audio() {
    let channel = agent_channel_for(Platform::Darwin, MISSING_PROGRAM.into());
    assert!(channel.open().await.is_ok());
}

#[tokio::test]
async fn linux_uses_the_avatar_audio_gate() {
    let channel = agent_channel_for(Platform::Linux, MISSING_PROGRAM.into());
    assert!(channel.open().await.is_err());
}

type Naming<'names> = (Platform, Option<&'names str>, [&'names str; 3]);

fn observed(configuration: &SpeechRuntimeConfiguration) -> Naming<'_> {
    let players = &configuration.players;
    (
        configuration.platform,
        configuration.avatar_audio_program.to_str(),
        [&players.paplay, &players.discord_voice, &players.seat_play].map(String::as_str),
    )
}

fn expected() -> Naming<'static> {
    (
        Platform::current(),
        Some("avatar-audio"),
        ["paplay", "discord-voice", "avatar-seat-play"],
    )
}

#[test]
fn configuration_names_the_programs_node_runs() -> TestResult {
    let configuration = SpeechRuntimeConfiguration::from_environment()?;
    assert_eq!(observed(&configuration), expected());
    Ok(())
}
