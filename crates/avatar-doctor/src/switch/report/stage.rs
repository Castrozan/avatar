use super::super::moment::{Boundary, Moments};

#[derive(Clone, Copy, Debug)]
pub struct Stage {
    pub name: &'static str,
    pub start: Boundary,
    pub end: Boundary,
}

pub const STAGES: [Stage; 14] = [
    Stage {
        name: "silence window",
        start: Boundary::SpeechEnded,
        end: Boundary::SegmentClosed,
    },
    Stage {
        name: "turn judgment and queue",
        start: Boundary::SegmentClosed,
        end: Boundary::TranscriptionStarted,
    },
    Stage {
        name: "transcription",
        start: Boundary::TranscriptionStarted,
        end: Boundary::TranscriptWritten,
    },
    Stage {
        name: "wake pickup",
        start: Boundary::TranscriptWritten,
        end: Boundary::WakeRead,
    },
    Stage {
        name: "turn assembly",
        start: Boundary::WakeRead,
        end: Boundary::DecisionStarted,
    },
    Stage {
        name: "decision API",
        start: Boundary::DecisionStarted,
        end: Boundary::DecisionDone,
    },
    Stage {
        name: "floor handoff",
        start: Boundary::DecisionDone,
        end: Boundary::PromptSent,
    },
    Stage {
        name: "prompt delivery",
        start: Boundary::PromptSent,
        end: Boundary::PromptDelivered,
    },
    Stage {
        name: "agent wake-up",
        start: Boundary::PromptDelivered,
        end: Boundary::AgentFirstCommand,
    },
    Stage {
        name: "reply composition",
        start: Boundary::AgentFirstCommand,
        end: Boundary::ReplyInvoked,
    },
    Stage {
        name: "speech dispatch",
        start: Boundary::ReplyInvoked,
        end: Boundary::SpeechReceived,
    },
    Stage {
        name: "speech queue",
        start: Boundary::SpeechReceived,
        end: Boundary::SynthesisStarted,
    },
    Stage {
        name: "synthesis",
        start: Boundary::SynthesisStarted,
        end: Boundary::Synthesized,
    },
    Stage {
        name: "playback start",
        start: Boundary::Synthesized,
        end: Boundary::FirstAudio,
    },
];

pub const END_TO_END: Stage = Stage {
    name: "end to end",
    start: Boundary::SpeechEnded,
    end: Boundary::FirstAudio,
};

#[must_use]
pub fn took(moments: &Moments, start: Boundary, end: Boundary) -> Option<f64> {
    Some(moments.get(end)? - moments.get(start)?)
}
