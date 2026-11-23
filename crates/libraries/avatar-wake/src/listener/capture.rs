use avatar_listening::speech_segmenter::SpeechSegmenter;

pub(super) const FRAMES_PER_ARM_CHECK: usize = 156;

#[derive(Debug, Default)]
pub struct Capture {
    pub(super) segmenter: SpeechSegmenter,
    pub(super) frames: usize,
}

impl Capture {
    pub(super) const fn arm_check_due(&self) -> bool {
        self.frames.is_multiple_of(FRAMES_PER_ARM_CHECK)
    }

    pub(super) fn heard(&mut self, frame: Vec<u8>) -> Option<Vec<u8>> {
        self.frames += 1;
        self.segmenter
            .feed(frame)
            .filter(|segment| !segment.is_empty())
    }
}
