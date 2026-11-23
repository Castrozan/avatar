use super::COMMAND_TIMEOUT_SECONDS;
use crate::talk::cli::arguments::OpenArguments;
use crate::talk::failure::TalkFailure;
use crate::talk::ports::Subprocess;
use crate::talk::ports::exec::run_checked;

pub fn show_chosen_character(
    process: &mut impl Subprocess,
    open: &OpenArguments,
) -> Result<(), TalkFailure> {
    let Some(character) = open.character.as_deref().filter(|name| !name.is_empty()) else {
        return Ok(());
    };
    let argv = ["avatar-command", "character", character];
    run_checked(process, &argv, Some(COMMAND_TIMEOUT_SECONDS)).map(|_| ())
}
