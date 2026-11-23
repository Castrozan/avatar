use std::io;

use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::effects::CLOSED_MESSAGE;
use avatar_listening::user_units::UnitEffectError;
use avatar_seat::edges::EdgeFailure;

use crate::failure::{Failure, FailureKind};

#[must_use]
pub fn os_failure(error: &io::Error) -> Failure {
    Failure::new(FailureKind::Os, &error.to_string())
}

#[must_use]
pub fn unit_failure(error: &UnitEffectError) -> Failure {
    let kind = match error {
        UnitEffectError::Command(_) | UnitEffectError::UnknownUnit(_) => FailureKind::Os,
        UnitEffectError::DidNotStop(_) => FailureKind::Runtime,
    };
    Failure::new(kind, &error.to_string())
}

#[must_use]
pub fn command_failure(command: &[String], error: &io::Error, seconds: u64) -> Failure {
    if error.kind() != io::ErrorKind::TimedOut {
        return os_failure(error);
    }
    let timed_out = EdgeFailure::TimedOut {
        command: command.to_vec(),
        seconds,
    };
    Failure::new(FailureKind::Subprocess, &timed_out.to_string())
}

#[must_use]
pub fn exit_failure(command: &[String], code: i32) -> Failure {
    let refused = EdgeFailure::NonZeroExit {
        command: command.to_vec(),
        code,
    };
    Failure::new(FailureKind::CalledProcess, &refused.to_string())
}

#[must_use]
pub fn edge_failure(error: &EdgeFailure) -> Failure {
    let kind = match error {
        EdgeFailure::Os(_) => FailureKind::Os,
        EdgeFailure::Malformed(_) => FailureKind::Value,
        EdgeFailure::TimedOut { .. } => FailureKind::Subprocess,
        EdgeFailure::NonZeroExit { .. } => FailureKind::CalledProcess,
    };
    Failure::new(kind, &error.to_string())
}

#[must_use]
pub fn talk_failure(error: &TalkFailure) -> Failure {
    let kind = if error.message() == CLOSED_MESSAGE {
        FailureKind::ConversationClosed
    } else {
        FailureKind::Runtime
    };
    Failure::new(kind, error.message())
}
