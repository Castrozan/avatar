use avatar_floor::floor::ServantCharacter;

use super::listing::ListedCharacters;
use crate::reach::CommandRunner;

pub const SWITCH_TIMEOUT_SECONDS: u64 = 30;

pub struct ListedServantCharacter<'a> {
    listed: ListedCharacters,
    runner: &'a dyn CommandRunner,
}

impl<'a> ListedServantCharacter<'a> {
    #[must_use]
    pub fn new(listed: ListedCharacters, runner: &'a dyn CommandRunner) -> Self {
        Self { listed, runner }
    }
}

impl ServantCharacter for ListedServantCharacter<'_> {
    fn show_for(&self, servant: &str) {
        switch_for(&self.listed, self.runner, servant);
    }
}

fn switch_for(listed: &ListedCharacters, runner: &dyn CommandRunner, servant: &str) {
    if let Some(character) = listed.character_for(servant) {
        let command = ["avatar-command", "character", character].map(str::to_owned);
        runner.stdout_of(&command, SWITCH_TIMEOUT_SECONDS);
    }
}
