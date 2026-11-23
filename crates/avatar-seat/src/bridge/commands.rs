use crate::support::words::joined_words;

const CLIENT_FLAG: &str = "--client-name=avatar-remote-seat";
const MICROPHONE_SINK: &str = "AvatarRemoteMicrophone";
const MEETING_SINK: &str = "AvatarRemoteMeeting";
const VOICE_SOURCE: &str = "AvatarLipSync.monitor";
const RAW_FORMAT: [&str; 2] = ["--raw", "--format=s16le"];
pub const MICROPHONE_RATE: u32 = 16000;
pub const MEETING_RATE: u32 = 16000;
pub const VOICE_RATE: u32 = 24000;
pub const VOICE_CHUNK_BYTES: usize = 960;

fn tool(head: &[&str], rate: u32, latency: u32) -> Vec<String> {
    let (rate_flag, latency_flag) = (
        format!("--rate={rate}"),
        format!("--latency-msec={latency}"),
    );
    joined_words(&[
        head,
        &RAW_FORMAT,
        &[&rate_flag, "--channels=1", &latency_flag],
    ])
}

fn playback(sink: &str, rate: u32) -> Vec<String> {
    let device = format!("--device={sink}");
    tool(&["pacat", "--playback", &device, CLIENT_FLAG], rate, 60)
}

#[must_use]
pub fn microphone_playback_command() -> Vec<String> {
    playback(MICROPHONE_SINK, MICROPHONE_RATE)
}

#[must_use]
pub fn meeting_playback_command() -> Vec<String> {
    playback(MEETING_SINK, MEETING_RATE)
}

#[must_use]
pub fn voice_capture_command() -> Vec<String> {
    let device = format!("--device={VOICE_SOURCE}");
    tool(&["parec", &device, CLIENT_FLAG], VOICE_RATE, 40)
}
