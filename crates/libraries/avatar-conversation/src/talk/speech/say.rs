use avatar_floor::quiet::Seat;

use super::spoken::{CompletedSpeech, spoken, spoken_in_turn};
use super::{SPEECH_TIMEOUT_SECONDS, mark_spoken};
use crate::talk::cli::arguments::SpeechArguments;
use crate::talk::failure::TalkFailure;
use crate::talk::ports::exec::run_checked;
use crate::talk::ports::{Boundaries, Console};
use crate::talk::state::seat::{current, speech_output};
use crate::voice_turns::{SpeechRequest, speech_command};

fn request_of(speech: &SpeechArguments) -> SpeechRequest {
    SpeechRequest {
        text: speech.text.clone(),
        emotion: speech.emotion.clone(),
        voice: speech.voice.clone(),
    }
}

pub fn speech_argv(
    boundary: &mut impl Boundaries,
    speech: &SpeechArguments,
) -> Result<Vec<String>, TalkFailure> {
    let seat = current(boundary)?;
    Ok(speech_command(&request_of(speech), speech_output(seat)))
}

fn print_and_exit(console: &mut impl Console, completed: &CompletedSpeech) -> i32 {
    console.print_out(&completed.stdout);
    completed.status
}

pub fn say(boundary: &mut impl Boundaries, speech: &SpeechArguments) -> Result<i32, TalkFailure> {
    mark_spoken(boundary);
    let argv = speech_argv(boundary, speech)?;
    let invoked = boundary.wall();
    let completed = spoken(boundary, argv, invoked)?;
    Ok(print_and_exit(boundary, &completed))
}

pub fn reply(boundary: &mut impl Boundaries, speech: &SpeechArguments) -> Result<i32, TalkFailure> {
    mark_spoken(boundary);
    let argv = speech_argv(boundary, speech)?;
    let completed = spoken_in_turn(boundary, argv)?;
    Ok(print_and_exit(boundary, &completed))
}

pub fn speak_on_seat(
    boundary: &mut impl Boundaries,
    text: &str,
    seat: Seat,
) -> Result<(), TalkFailure> {
    mark_spoken(boundary);
    let request = SpeechRequest {
        text: text.to_owned(),
        emotion: "neutral".to_owned(),
        voice: None,
    };
    let argv = speech_command(&request, speech_output(seat));
    run_checked(boundary, &argv, Some(SPEECH_TIMEOUT_SECONDS)).map(|_| ())
}
