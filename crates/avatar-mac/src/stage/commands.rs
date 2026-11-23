use super::ports::Stage;
use super::{COMMAND_TIMEOUT_SECONDS, SPEAKERS, SPEECH_TIMEOUT_SECONDS};
use crate::failure::Failure;

fn words(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

pub fn speak(
    stage: &mut impl Stage,
    text: &str,
    emotion: &str,
    voice: Option<&str>,
) -> Result<(), Failure> {
    let mut arguments = words(&["speak", text, emotion, SPEAKERS]);
    arguments.extend(voice.filter(|name| !name.is_empty()).map(str::to_owned));
    stage.avatar_command(&arguments, SPEECH_TIMEOUT_SECONDS)
}

pub fn show_character(stage: &mut impl Stage, character: Option<&str>) -> Result<(), Failure> {
    match character.filter(|chosen| !chosen.is_empty()) {
        Some(chosen) => {
            stage.avatar_command(&words(&["character", chosen]), COMMAND_TIMEOUT_SECONDS)
        }
        None => Ok(()),
    }
}

pub fn present(stage: &mut impl Stage, shown: bool) -> Result<(), Failure> {
    let visibility = if shown { "shown" } else { "hidden" };
    stage.avatar_command(&words(&["presence", visibility]), COMMAND_TIMEOUT_SECONDS)
}
