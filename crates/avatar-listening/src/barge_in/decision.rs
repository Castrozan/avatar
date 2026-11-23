use crate::numeric::count_as_f64;
use crate::speech_segmenter::FRAME_SECONDS;

pub const SPEECH_PROBABILITY: f64 = 0.85;
pub const SPEECH_FRAMES: usize = 5;

pub trait Reply {
    fn speaking(&self) -> bool;
    fn stop(&mut self);
}

pub trait VoiceActivity {
    fn reset(&mut self);
    fn speech_probability(&mut self, frame: &[u8]) -> f64;
}

pub struct BargeIn<Speaker, Activity> {
    reply: Speaker,
    voice_activity: Activity,
    hearing_reply: bool,
    segment_frames: usize,
    speech_frames: usize,
    stopped: bool,
}

impl<Speaker: Reply, Activity: VoiceActivity> BargeIn<Speaker, Activity> {
    #[must_use]
    pub fn new(reply: Speaker, voice_activity: Activity) -> Self {
        Self {
            reply,
            voice_activity,
            hearing_reply: false,
            segment_frames: 0,
            speech_frames: 0,
            stopped: false,
        }
    }

    pub fn begin_segment(&mut self) {
        self.segment_frames = 0;
        self.speech_frames = 0;
        self.stopped = false;
    }

    pub fn hear(&mut self, frame: &[u8], loud: bool) -> Option<f64> {
        hear_frame(self, frame, loud)
    }
}

fn hear_frame<Speaker: Reply, Activity: VoiceActivity>(
    barge: &mut BargeIn<Speaker, Activity>,
    frame: &[u8],
    loud: bool,
) -> Option<f64> {
    barge.segment_frames += 1;
    if !barge.reply.speaking() {
        barge.hearing_reply = false;
        return None;
    }
    hear_while_speaking(barge, frame, loud)
}

fn hear_while_speaking<Speaker: Reply, Activity: VoiceActivity>(
    barge: &mut BargeIn<Speaker, Activity>,
    frame: &[u8],
    loud: bool,
) -> Option<f64> {
    let heard = speech_heard(barge, frame);
    barge.speech_frames += usize::from(heard && loud);
    if barge.speech_frames >= SPEECH_FRAMES && !barge.stopped {
        return Some(stop_reply(barge));
    }
    None
}

fn speech_heard<Speaker: Reply, Activity: VoiceActivity>(
    barge: &mut BargeIn<Speaker, Activity>,
    frame: &[u8],
) -> bool {
    if !barge.hearing_reply {
        barge.voice_activity.reset();
        barge.hearing_reply = true;
    }
    barge.voice_activity.speech_probability(frame) >= SPEECH_PROBABILITY
}

fn stop_reply<Speaker: Reply, Activity: VoiceActivity>(
    barge: &mut BargeIn<Speaker, Activity>,
) -> f64 {
    barge.stopped = true;
    barge.reply.stop();
    count_as_f64(barge.segment_frames) * FRAME_SECONDS
}
