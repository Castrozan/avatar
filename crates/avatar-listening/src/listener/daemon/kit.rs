use crate::barge_in::decision::{BargeIn, Reply, VoiceActivity};
use crate::listener::capture::{BargeInHearing, CaptureLauncher};
use crate::turn_detector::TurnFinishedJudge;

pub trait ListenerKit: Sync {
    type Launcher: CaptureLauncher;
    type Judge: TurnFinishedJudge;
    type Hearing: BargeInHearing;

    fn launcher(&self) -> Self::Launcher;
    fn judge(&self) -> Self::Judge;
    fn hearing(&self) -> Self::Hearing;
}

pub fn barge_in_hearing<Speaker: Reply, Activity: VoiceActivity>(
    reply: Speaker,
    voice_activity: Activity,
) -> BargeIn<Speaker, Activity> {
    BargeIn::new(reply, voice_activity)
}
