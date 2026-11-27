#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReportKind {
    Progress,
    Answer,
    Failure(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentReport {
    pub text: String,
    pub seconds: f64,
    pub kind: ReportKind,
}

impl ReportKind {
    #[must_use]
    pub fn is_final(&self) -> bool {
        *self != Self::Progress
    }

    #[must_use]
    pub fn into_failure(self) -> Option<String> {
        match self {
            Self::Failure(reason) => Some(reason),
            Self::Progress | Self::Answer => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentRequest {
    pub holder: avatar_floor::addressee::Addressee,
    pub question: AgentQuestion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentQuestion {
    pub request: String,
    pub number: u32,
}

impl AgentQuestion {
    #[must_use]
    pub fn answer_tag(&self) -> String {
        format!("{}-{}", super::brief::ANSWER_TAG, self.number)
    }
}
