use serde_json::Value;

use crate::names::vocabulary::Vocabulary;
use crate::protocol::request::Operation;

#[must_use]
pub fn perform(operation: Operation, vocabulary: Option<&Vocabulary>) -> Value {
    match operation {
        Operation::Folded(request) => request.answer(),
        Operation::SoundKey(request) => request.answer(),
        Operation::Ratio(request) => request.answer(),
        Operation::WordMatches(request) => request.answer(),
        Operation::OpensWithGreeting(request) => request.answer(),
        other => perform_hearing(other, vocabulary),
    }
}

fn perform_hearing(operation: Operation, vocabulary: Option<&Vocabulary>) -> Value {
    match operation {
        Operation::HeardName(request) => request.answer(vocabulary),
        Operation::SessionsHeardAnywhere(request) => request.sessions_heard_anywhere(vocabulary),
        Operation::VocativeSession(request) => request.vocative_session(vocabulary),
        Operation::AddressedSession(request) => request.addressed_session(vocabulary),
        Operation::NamedInTurn(request) => request.named_in_turn(vocabulary),
        other => perform_decision(other),
    }
}

fn perform_decision(operation: Operation) -> Value {
    match operation {
        Operation::DecisionRequest(request) => request.answer(),
        Operation::DecisionVerdict(request) => request.answer(),
        _ => Value::Null,
    }
}
