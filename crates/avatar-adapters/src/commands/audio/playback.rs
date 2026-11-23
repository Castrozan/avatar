use crate::commands::Programs;

const SAMPLE_WIDTH_BYTES: usize = 2;
const CHUNKS_PER_SECOND: u32 = 10;
const PLAYBACK_LATENCY_MILLISECONDS: u32 = 80;
const CAPTURE_LATENCY_MILLISECONDS: u32 = 40;
const LIP_SYNC_SINK: &str = "AvatarLipSync";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomSink {
    Default,
    LipSync,
}

impl RoomSink {
    const fn device(self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::LipSync => Some(LIP_SYNC_SINK),
        }
    }

    const fn client(self) -> &'static str {
        match self {
            Self::Default => "avatar-live",
            Self::LipSync => "avatar-live-lip-sync",
        }
    }
}

#[must_use]
pub(super) fn chunk_bytes(rate: u32) -> usize {
    (rate / CHUNKS_PER_SECOND) as usize * SAMPLE_WIDTH_BYTES
}

pub(super) fn playback_command(programs: &Programs, rate: u32, sink: RoomSink) -> Vec<String> {
    let mut words = programs.words(&["pacat", "--playback", "--raw"]);
    words.extend([
        format!("--rate={rate}"),
        "--channels=1".to_owned(),
        "--format=s16le".to_owned(),
        format!("--latency-msec={PLAYBACK_LATENCY_MILLISECONDS}"),
        format!("--client-name={}", sink.client()),
    ]);
    words.extend(sink.device().map(|device| format!("--device={device}")));
    words
}

pub(super) fn capture_command(programs: &Programs, rate: u32, device: &str) -> Vec<String> {
    let mut words = programs.words(&["parec"]);
    words.push(format!("--device={device}"));
    words.extend(["--raw".to_owned(), format!("--rate={rate}")]);
    words.extend(["--channels=1", "--format=s16le"].map(str::to_owned));
    words.push(format!("--latency-msec={CAPTURE_LATENCY_MILLISECONDS}"));
    words.push("--client-name=avatar-live-microphone".to_owned());
    words
}
