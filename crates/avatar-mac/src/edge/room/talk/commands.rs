use avatar_conversation::talk::cli::arguments::{
    SpeechArguments, TalkArguments, TalkCommand, WaitArguments, parse_talk_arguments,
};
use avatar_conversation::talk::hearing::{follow, hear};
use avatar_conversation::talk::ports::Boundaries;
use avatar_conversation::talk::state::addressee::holder_other_than;
use avatar_listening::conversation_state::command_of;

use crate::edge::failure::talk_failure;
use crate::edge::room::MacRoom;
use crate::failure::{Failure, FailureKind};
use crate::talk::{RoomCommands, TalkRequest};

fn speech_of(command: &TalkCommand) -> Option<&SpeechArguments> {
    match command {
        TalkCommand::Say(speech) => Some(speech),
        TalkCommand::Ask(asking) => Some(&asking.speech),
        _ => None,
    }
}

fn wait_of(command: &TalkCommand) -> f64 {
    match command {
        TalkCommand::Ask(asking) => asking.wait.wait,
        TalkCommand::Hear(hearing) => hearing.wait,
        _ => 0.0,
    }
}

fn with_speech(mut request: TalkRequest, command: &TalkCommand) -> TalkRequest {
    if let Some(said) = speech_of(command) {
        request.text.clone_from(&said.text);
        request.emotion.clone_from(&said.emotion);
        request.voice.clone_from(&said.voice);
    }
    request
}

fn request_of(arguments: &TalkArguments) -> TalkRequest {
    let open = arguments.open();
    let command = command_of(open.live, arguments.command.conversation_command());
    let mut request = TalkRequest::new(command);
    request.language = open.language;
    request.character = open.character;
    request.agent = open.agent;
    request.wait = wait_of(&arguments.command);
    with_speech(request, &arguments.command)
}

fn usage_failure(error: impl std::fmt::Display) -> Failure {
    Failure::new(FailureKind::Value, error.to_string().trim_end())
}

impl<Boundary: Boundaries> RoomCommands for MacRoom<Boundary> {
    fn parse_request(&mut self, arguments: &[String]) -> Result<TalkRequest, Failure> {
        let parsed = parse_talk_arguments(arguments);
        parsed.map(|talk| request_of(&talk)).map_err(usage_failure)
    }

    fn hear(&mut self, request: &TalkRequest) -> Result<(), Failure> {
        let waiting = WaitArguments { wait: request.wait };
        hear(&mut self.talk, &waiting).map_err(|error| talk_failure(&error))
    }

    fn follow(&mut self, _request: &TalkRequest) -> Result<(), Failure> {
        follow(&mut self.talk).map_err(|error| talk_failure(&error))
    }

    fn holder_other_than(&mut self, pane: Option<&str>) -> Option<String> {
        let other = holder_other_than(&mut self.talk, pane).ok().flatten();
        other.map(|holder| holder.servant)
    }
}
