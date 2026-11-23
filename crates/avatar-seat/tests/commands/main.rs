use avatar_seat::bridge::{
    VOICE_CHUNK_BYTES, meeting_playback_command, microphone_playback_command, voice_capture_command,
};
use avatar_seat::local::{SeatConfig, tunnel_command};
use serde_json::Value;

fn oracle() -> Value {
    serde_json::from_str(include_str!("fixtures/commands.json")).unwrap_or(Value::Null)
}

fn expected(key: &str) -> Vec<String> {
    serde_json::from_value(oracle()[key].clone()).unwrap_or_default()
}

#[test]
fn the_audio_tools_are_started_as_the_python_started_them() {
    assert_eq!(microphone_playback_command(), expected("microphone"));
    assert_eq!(voice_capture_command(), expected("voice"));
    assert_eq!(meeting_playback_command(), expected("meeting"));
    assert_eq!(oracle()["constants"]["voice_chunk"], VOICE_CHUNK_BYTES);
}

#[test]
fn the_tunnel_carries_the_seat_the_window_and_the_controls() {
    let config = SeatConfig {
        avatar_host: "chise".to_owned(),
        browser_application: "/nix/store/browser/Google Chrome for Testing.app".to_owned(),
        home: "<home>".to_owned(),
        python: "<python>".to_owned(),
        holder_path: "<holder>".to_owned(),
    };
    assert_eq!(tunnel_command(&config), expected("tunnel"));
    assert_eq!(config.browser_name(), oracle()["constants"]["browser"]);
}
