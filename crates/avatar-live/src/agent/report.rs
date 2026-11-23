#[derive(Clone, Debug, PartialEq)]
pub struct AgentReport {
    pub text: String,
    pub seconds: f64,
    pub is_final: bool,
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
