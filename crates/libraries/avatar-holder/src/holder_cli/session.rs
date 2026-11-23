use crate::default_holder::DefaultHolder;
use crate::floor_holder::HoldingInterruption;
use crate::holder_cli::commands::{
    CLOSE_TIMEOUT_SECONDS, CommandRunner, OPEN_TIMEOUT_SECONDS, close_command, open_command,
};

pub const CONVERSATION_NOT_OURS: &str =
    "A conversation was already open without this holder; it was left alone";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HolderEnd {
    Closed,
    Stopped,
    Failed(String),
}

pub trait ConversationSession {
    fn configured_holder(&mut self) -> Result<Option<DefaultHolder>, String>;
    fn hold_floor(&mut self, holder: &DefaultHolder) -> HoldingInterruption;
}

#[must_use]
pub fn hold_the_conversation(
    commands: &mut impl CommandRunner,
    session: &mut impl ConversationSession,
    holder: &DefaultHolder,
) -> HolderEnd {
    match commands.run_command(&open_command(holder), OPEN_TIMEOUT_SECONDS) {
        Ok(()) => hold_when_configured(session, holder),
        Err(failure) => HolderEnd::Failed(failure.message().to_owned()),
    }
}

fn hold_when_configured(
    session: &mut impl ConversationSession,
    holder: &DefaultHolder,
) -> HolderEnd {
    match session.configured_holder() {
        Ok(configured) if configured.as_ref() == Some(holder) => {
            end_of_floor(session.hold_floor(holder))
        }
        Ok(_) => HolderEnd::Failed(CONVERSATION_NOT_OURS.to_owned()),
        Err(message) => HolderEnd::Failed(message),
    }
}

fn end_of_floor(interruption: HoldingInterruption) -> HolderEnd {
    match interruption {
        HoldingInterruption::ConversationClosed => HolderEnd::Closed,
        HoldingInterruption::Stopped => HolderEnd::Stopped,
    }
}

pub fn conclude(end: HolderEnd, commands: &mut impl CommandRunner) -> Result<(), String> {
    match end {
        HolderEnd::Closed => Ok(()),
        HolderEnd::Stopped => {
            let _ = commands.run_command(&close_command(), CLOSE_TIMEOUT_SECONDS);
            Ok(())
        }
        HolderEnd::Failed(message) => Err(message),
    }
}
