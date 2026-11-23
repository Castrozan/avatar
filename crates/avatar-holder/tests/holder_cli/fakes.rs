use avatar_holder::default_holder::DefaultHolder;
use avatar_holder::floor_holder::HoldingInterruption;
use avatar_holder::holder_cli::{CommandFailure, CommandRunner, ConversationSession};

#[derive(Default)]
pub(crate) struct RecordingCommands {
    pub(crate) failure: Option<String>,
    pub(crate) commands: Vec<(Vec<String>, u64)>,
}

impl CommandRunner for RecordingCommands {
    fn run_command(
        &mut self,
        command: &[String],
        timeout_seconds: u64,
    ) -> Result<(), CommandFailure> {
        self.commands.push((command.to_vec(), timeout_seconds));
        match &self.failure {
            Some(message) => Err(CommandFailure::new(message.clone())),
            None => Ok(()),
        }
    }
}

pub(crate) struct ScriptedSession {
    pub(crate) configured: Result<Option<DefaultHolder>, String>,
    pub(crate) interruption: HoldingInterruption,
    pub(crate) held: usize,
}

impl ScriptedSession {
    pub(crate) fn configured_as(configured: Option<DefaultHolder>) -> Self {
        Self {
            configured: Ok(configured),
            interruption: HoldingInterruption::ConversationClosed,
            held: 0,
        }
    }
}

impl ConversationSession for ScriptedSession {
    fn configured_holder(&mut self) -> Result<Option<DefaultHolder>, String> {
        self.configured.clone()
    }

    fn hold_floor(&mut self, _holder: &DefaultHolder) -> HoldingInterruption {
        self.held += 1;
        self.interruption
    }
}
