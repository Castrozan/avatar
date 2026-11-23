use avatar_conversation::talk::hearing::listen_after;
use avatar_conversation::talk::ports::Boundaries;
use avatar_conversation::talk::speech::{CompletedSpeech, mark_spoken, spoken_in_turn};
use avatar_conversation::talk::state::addressee::addresses;
use avatar_conversation::talk::state::seat::speech_output;
use avatar_conversation::talk::state::this_pane;
use avatar_conversation::voice_turns::{SpeechRequest, speech_command};
use avatar_floor::quiet::Seat;

use crate::edge::failure::talk_failure;
use crate::edge::room::MacRoom;
use crate::failure::Failure;
use crate::talk::TalkRequest;
use crate::turns::RoomTurns;

impl<Boundary: Boundaries> RoomTurns for MacRoom<Boundary> {
    type Command = Vec<String>;
    type Completed = CompletedSpeech;

    fn this_pane(&mut self) -> Option<String> {
        this_pane(&self.talk)
    }

    fn room_addresses(&mut self, pane: Option<&str>) -> bool {
        addresses(&mut self.talk, pane).unwrap_or(false)
    }

    fn mark_spoken(&mut self) -> Result<(), Failure> {
        mark_spoken(&mut self.talk);
        Ok(())
    }

    fn speech_command_on_seat(&mut self, request: &TalkRequest) -> Vec<String> {
        let speech = SpeechRequest {
            text: request.text.clone(),
            emotion: request.emotion.clone(),
            voice: request.voice.clone(),
        };
        speech_command(&speech, speech_output(Seat::Room))
    }

    fn spoken_in_turn(&mut self, command: &Vec<String>) -> Result<CompletedSpeech, Failure> {
        spoken_in_turn(&mut self.talk, command.clone()).map_err(|error| talk_failure(&error))
    }

    fn print_and_exit(&mut self, completed: CompletedSpeech) -> i32 {
        self.talk.print_out(&completed.stdout);
        completed.status
    }

    fn listen_after(&mut self, completed: &CompletedSpeech, wait: f64) -> Result<(), Failure> {
        listen_after(&mut self.talk, completed, wait).map_err(|error| talk_failure(&error))
    }

    fn acquire_consumer(&mut self) -> Result<(), Failure> {
        self.talk.acquire().map_err(|error| talk_failure(&error))
    }

    fn release_consumer(&mut self) {
        self.talk.release();
    }
}
