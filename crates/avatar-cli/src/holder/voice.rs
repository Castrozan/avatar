use std::io::Stderr;

use avatar_adapters::commands::Programs;
use avatar_adapters::holder::{HermesReplies, HermesTurns, LineReporter, SeatSpeech, local_day};
use avatar_conversation::talk::state::seat::{current, speech_output};
use avatar_holder::floor_holder::{ReplySource, Reporter, Speech, SpeechFailure};
use avatar_holder::hermes_turn::HermesTurnError;
use std::path::Path;

use crate::talk::TalkBoundaries;

pub(super) struct HolderVoice {
    replies: HermesReplies<HermesTurns, fn() -> String>,
    speech: SeatSpeech<Box<dyn FnMut() -> String>>,
    reporter: LineReporter<Stderr>,
}

fn seat_output(directory: &Path) -> Box<dyn FnMut() -> String> {
    let mut boundaries = TalkBoundaries::on_this_machine(directory.to_path_buf());
    Box::new(move || {
        let seat = current(&mut boundaries).unwrap_or(avatar_floor::quiet::Seat::Room);
        speech_output(seat).to_owned()
    })
}

impl HolderVoice {
    pub(super) fn new(profile: &str, directory: &Path) -> Self {
        let programs = Programs::on_path();
        Self {
            replies: HermesReplies::new(HermesTurns::new(programs.clone()), profile, local_day),
            speech: SeatSpeech::new(directory, programs, seat_output(directory)),
            reporter: LineReporter::new(std::io::stderr()),
        }
    }
}

impl ReplySource for HolderVoice {
    fn reply_to(&mut self, heard: &str) -> Result<String, HermesTurnError> {
        self.replies.reply_to(heard)
    }
}

impl Speech for HolderVoice {
    fn speak(&mut self, text: &str) -> Result<(), SpeechFailure> {
        self.speech.speak(text)
    }
}

impl Reporter for HolderVoice {
    fn report(&mut self, message: &str) {
        self.reporter.report(message);
    }
}
