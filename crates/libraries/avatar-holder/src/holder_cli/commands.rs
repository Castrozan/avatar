use crate::default_holder::DefaultHolder;

pub const OPEN_TIMEOUT_SECONDS: u64 = 180;
pub const CLOSE_TIMEOUT_SECONDS: u64 = 120;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFailure {
    message: String,
}

impl CommandFailure {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

pub trait CommandRunner {
    fn run_command(
        &mut self,
        command: &[String],
        timeout_seconds: u64,
    ) -> Result<(), CommandFailure>;
}

#[must_use]
pub fn open_command(holder: &DefaultHolder) -> Vec<String> {
    let mut command = ["avatar-talk", "open", "--seat", "discord", "--holder"]
        .map(str::to_owned)
        .to_vec();
    command.push(holder.profile().to_owned());
    command.extend(character_flag(holder));
    command
}

fn character_flag(holder: &DefaultHolder) -> Vec<String> {
    holder
        .character()
        .map(|character| vec!["--character".to_owned(), character.to_owned()])
        .unwrap_or_default()
}

#[must_use]
pub fn close_command() -> Vec<String> {
    ["avatar-talk", "close"].map(str::to_owned).to_vec()
}
