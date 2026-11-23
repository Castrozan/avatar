use std::io;

use avatar_room_audio::mouth::{MouthLevels, VoiceLevelSink};
use avatar_room_audio::pcm::encode_samples;

const CONSTRUCTED_AT: f64 = 100.0;

struct BrokenSink;

impl VoiceLevelSink for BrokenSink {
    fn send_text(&mut self, _text: &str) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::BrokenPipe))
    }

    fn close(&mut self) {}
}

#[test]
fn a_broken_control_connection_is_reported_not_swallowed() {
    let mut levels = MouthLevels::new(BrokenSink, CONSTRUCTED_AT);
    levels.show(&encode_samples(&[16384, -16384]), CONSTRUCTED_AT);
    assert!(levels.deliver_due(CONSTRUCTED_AT + 1.0).is_err());
}
