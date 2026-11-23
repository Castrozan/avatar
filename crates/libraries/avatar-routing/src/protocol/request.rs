use serde::Deserialize;

use crate::names::vocabulary::Vocabulary;
use crate::protocol::decision_operations::{DecisionPromptRequest, DecisionVerdictRequest};
use crate::protocol::hearing_operations::{HeardNameRequest, SessionHearingRequest};
use crate::protocol::text_operations::{
    FoldedRequest, OpensWithGreetingRequest, RatioRequest, SoundKeyRequest, WordMatchesRequest,
};

#[derive(Deserialize)]
pub struct Request {
    pub vocabulary: Option<Vocabulary>,
    #[serde(flatten)]
    pub operation: Operation,
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Operation {
    Folded(FoldedRequest),
    SoundKey(SoundKeyRequest),
    Ratio(RatioRequest),
    WordMatches(WordMatchesRequest),
    OpensWithGreeting(OpensWithGreetingRequest),
    HeardName(HeardNameRequest),
    SessionsHeardAnywhere(SessionHearingRequest),
    VocativeSession(SessionHearingRequest),
    AddressedSession(SessionHearingRequest),
    NamedInTurn(SessionHearingRequest),
    DecisionRequest(DecisionPromptRequest),
    DecisionVerdict(DecisionVerdictRequest),
}
