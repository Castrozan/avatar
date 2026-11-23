#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary {
    SpeechEnded,
    SegmentClosed,
    TranscriptionStarted,
    TranscriptWritten,
    WakeRead,
    DecisionStarted,
    DecisionDone,
    PromptSent,
    PromptDelivered,
    AgentFirstCommand,
    ReplyInvoked,
    SpeechReceived,
    SynthesisStarted,
    Synthesized,
    FirstAudio,
}

const KEYS: [&str; 15] = [
    "speech_ended",
    "segment_closed",
    "transcription_started",
    "transcript_written",
    "wake_read",
    "decision_started",
    "decision_done",
    "prompt_sent",
    "prompt_delivered",
    "agent_first_command",
    "reply_invoked",
    "speech_received",
    "synthesis_started",
    "synthesized",
    "first_audio",
];

impl Boundary {
    pub const ALL: [Self; 15] = [
        Self::SpeechEnded,
        Self::SegmentClosed,
        Self::TranscriptionStarted,
        Self::TranscriptWritten,
        Self::WakeRead,
        Self::DecisionStarted,
        Self::DecisionDone,
        Self::PromptSent,
        Self::PromptDelivered,
        Self::AgentFirstCommand,
        Self::ReplyInvoked,
        Self::SpeechReceived,
        Self::SynthesisStarted,
        Self::Synthesized,
        Self::FirstAudio,
    ];

    #[must_use]
    pub fn key(self) -> &'static str {
        KEYS[usize::from(self as u8)]
    }

    #[must_use]
    pub fn label(self) -> String {
        self.key().replace('_', " ")
    }
}
