use crate::barge_in::decision::{BargeIn, Reply, VoiceActivity};

pub trait BargeInHearing {
    fn begin_segment(&mut self);
    fn hear(&mut self, frame: &[u8], loud: bool);
}

pub struct NoBargeIn;

impl BargeInHearing for NoBargeIn {
    fn begin_segment(&mut self) {}

    fn hear(&mut self, _frame: &[u8], _loud: bool) {}
}

impl<Speaker: Reply, Activity: VoiceActivity> BargeInHearing for BargeIn<Speaker, Activity> {
    fn begin_segment(&mut self) {
        BargeIn::begin_segment(self);
    }

    fn hear(&mut self, frame: &[u8], loud: bool) {
        BargeIn::hear(self, frame, loud);
    }
}
