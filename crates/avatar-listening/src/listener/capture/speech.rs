use super::hearing::BargeInHearing;
use super::source::FrameSink;
use crate::speech_segmenter::SpeechSegmenter;
use crate::turn_detector::{TurnFinishedJudge, TurnTracker};

pub const FRAME_BYTES: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeardMoments {
    pub speech_ended: f64,
    pub segment_closed: f64,
}

pub trait SegmentDesk {
    fn begin_pending(&mut self);
    fn finish_pending(&mut self);
    fn enqueue(
        &mut self,
        source: &str,
        segment: Vec<u8>,
        turn_finished: bool,
        heard: HeardMoments,
    ) -> bool;
}

pub struct SourceSpeech<Judge, Hearing, Desk> {
    source: String,
    segmenter: SpeechSegmenter,
    turn: TurnTracker<Judge>,
    hearing: Hearing,
    desk: Desk,
    now: fn() -> f64,
}

impl<Judge: TurnFinishedJudge, Hearing: BargeInHearing, Desk: SegmentDesk>
    SourceSpeech<Judge, Hearing, Desk>
{
    pub fn new(source: &str, judge: Judge, hearing: Hearing, desk: Desk, now: fn() -> f64) -> Self {
        Self {
            source: source.to_owned(),
            segmenter: SpeechSegmenter::default(),
            turn: TurnTracker::new(judge),
            hearing,
            desk,
            now,
        }
    }

    pub fn feed(&mut self, frame: &[u8]) {
        let was_active = self.segmenter.active();
        let segment = self.segmenter.feed(frame.to_vec());
        (self.segmenter.active() && !was_active).then(|| self.begin_segment());
        self.hearing.hear(frame, self.segmenter.loud());
        (was_active && !self.segmenter.active()).then(|| self.hand_over(segment));
    }

    fn begin_segment(&mut self) {
        self.desk.begin_pending();
        self.hearing.begin_segment();
    }

    fn hand_over(&mut self, segment: Option<Vec<u8>>) {
        match segment {
            Some(segment) => self.enqueue_segment(segment),
            None => self.desk.finish_pending(),
        }
    }

    fn enqueue_segment(&mut self, segment: Vec<u8>) {
        let closed = (self.now)();
        let speech_ended = closed - self.segmenter.closing_silence_seconds();
        let heard = HeardMoments {
            speech_ended,
            segment_closed: closed,
        };
        let finished = self.turn.add(&segment);
        let accepted = self.desk.enqueue(&self.source, segment, finished, heard);
        (!accepted).then(|| self.desk.finish_pending());
    }

    pub fn abandon(&mut self) {
        self.segmenter.active().then(|| self.desk.finish_pending());
    }
}

impl<Judge: TurnFinishedJudge, Hearing: BargeInHearing, Desk: SegmentDesk> FrameSink
    for SourceSpeech<Judge, Hearing, Desk>
{
    fn feed(&mut self, frame: &[u8]) {
        SourceSpeech::feed(self, frame);
    }

    fn abandon(&mut self) {
        SourceSpeech::abandon(self);
    }
}
