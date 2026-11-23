use std::collections::VecDeque;

use crate::numeric::count_as_f64;
use crate::pcm::frame_level;

pub const FRAME_SECONDS: f64 = 0.032;
pub const SPEECH_LEVEL: f64 = 300.0;
pub const LEAD_IN_FRAMES: usize = 9;
pub const TRAILING_SILENCE_FRAMES: usize = 21;
pub const MINIMUM_SPEECH_FRAMES: usize = 9;
pub const MAXIMUM_SEGMENT_FRAMES: usize = 375;

#[derive(Debug, Default)]
pub struct SpeechSegmenter {
    lead_in: VecDeque<Vec<u8>>,
    segment: Vec<Vec<u8>>,
    speech_frames: usize,
    silent_frames: usize,
    closing_silence_seconds: f64,
    loud: bool,
}

impl SpeechSegmenter {
    #[must_use]
    pub fn active(&self) -> bool {
        !self.segment.is_empty()
    }

    #[must_use]
    pub fn loud(&self) -> bool {
        self.loud
    }

    #[must_use]
    pub fn closing_silence_seconds(&self) -> f64 {
        self.closing_silence_seconds
    }

    pub fn feed(&mut self, frame: Vec<u8>) -> Option<Vec<u8>> {
        feed_frame(self, frame)
    }
}

fn feed_frame(segmenter: &mut SpeechSegmenter, frame: Vec<u8>) -> Option<Vec<u8>> {
    let speaking = frame_level(&frame) >= SPEECH_LEVEL;
    segmenter.loud = speaking;
    if segmenter.segment.is_empty() {
        wait_for_speech(segmenter, frame, speaking);
        return None;
    }
    continue_segment(segmenter, frame, speaking)
}

fn wait_for_speech(segmenter: &mut SpeechSegmenter, frame: Vec<u8>, speaking: bool) {
    if !speaking {
        remember_lead_in(segmenter, frame);
        return;
    }
    segmenter.segment = segmenter.lead_in.drain(..).collect();
    segmenter.segment.push(frame);
    segmenter.speech_frames = 1;
    segmenter.silent_frames = 0;
}

fn remember_lead_in(segmenter: &mut SpeechSegmenter, frame: Vec<u8>) {
    segmenter.lead_in.push_back(frame);
    if segmenter.lead_in.len() > LEAD_IN_FRAMES {
        segmenter.lead_in.pop_front();
    }
}

fn continue_segment(
    segmenter: &mut SpeechSegmenter,
    frame: Vec<u8>,
    speaking: bool,
) -> Option<Vec<u8>> {
    segmenter.segment.push(frame);
    count_speech(segmenter, speaking);
    if segment_ends(segmenter) {
        return finish(segmenter);
    }
    None
}

fn count_speech(segmenter: &mut SpeechSegmenter, speaking: bool) {
    if speaking {
        segmenter.speech_frames += 1;
        segmenter.silent_frames = 0;
    } else {
        segmenter.silent_frames += 1;
    }
}

fn segment_ends(segmenter: &SpeechSegmenter) -> bool {
    segmenter.silent_frames >= TRAILING_SILENCE_FRAMES
        || segmenter.segment.len() >= MAXIMUM_SEGMENT_FRAMES
}

fn finish(segmenter: &mut SpeechSegmenter) -> Option<Vec<u8>> {
    let was_speech = segmenter.speech_frames >= MINIMUM_SPEECH_FRAMES;
    segmenter.closing_silence_seconds = count_as_f64(segmenter.silent_frames) * FRAME_SECONDS;
    let segment: Vec<u8> = std::mem::take(&mut segmenter.segment).concat();
    segmenter.speech_frames = 0;
    segmenter.silent_frames = 0;
    was_speech.then_some(segment)
}
