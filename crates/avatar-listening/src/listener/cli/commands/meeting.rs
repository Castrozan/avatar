use serde_json::{Map, Value, json};

use crate::conversation_state::{
    ConversationAction, ConversationCommand, ListenerPurpose, decide, observe,
};
use crate::listener::cli::arguments::{MeetingSeat, StartArguments};
use crate::listener::cli::failure::ListenFailure;
use crate::listener::cli::ports::ListenPorts;
use crate::listener::cli::python_json::python_json;
use crate::listener::cli::sources::{remote_seat_sources, selected_sources};
use crate::listener::session::{SessionDirectory, SessionRequest, begin_listening, live_purpose};

const CONVERSATION_TARGET: &str = "avatar-conversation.target";
const CONVERSATION_OPEN: &str =
    "A conversation is open; run avatar-talk close before transcribing a meeting";

type Fields = Map<String, Value>;

fn refuse_open_conversation(
    directory: &SessionDirectory,
    ports: &mut ListenPorts,
) -> Result<(), ListenFailure> {
    let target_active = ports.units.is_active(CONVERSATION_TARGET);
    let phase = observe(target_active, live_purpose(directory, &mut ports.units));
    let action = decide(phase, ConversationCommand::TranscribeMeeting, None, None);
    if action == ConversationAction::RefuseConversation {
        return Err(ListenFailure::refused(CONVERSATION_OPEN));
    }
    Ok(())
}

fn seat_sources(
    arguments: &StartArguments,
    ports: &mut ListenPorts,
) -> Result<Fields, ListenFailure> {
    let microphone = arguments.microphone.as_deref();
    match arguments.seat {
        MeetingSeat::Remote => remote_seat_sources(microphone),
        MeetingSeat::Room => selected_sources(microphone, ports.pulse),
    }
}

fn settings_of(arguments: &StartArguments) -> Fields {
    let mut settings = Map::new();
    if arguments.seat == MeetingSeat::Remote {
        settings.insert("seat".into(), "remote".into());
    }
    settings.insert("language".into(), json!(arguments.language));
    settings.insert("identify_speakers".into(), json!(true));
    settings
}

fn request_of(
    arguments: &StartArguments,
    ports: &mut ListenPorts,
) -> Result<SessionRequest, ListenFailure> {
    let sources = seat_sources(arguments, ports)?;
    let settings = settings_of(arguments);
    Ok(SessionRequest {
        purpose: ListenerPurpose::Meeting,
        sources,
        settings,
    })
}

pub(super) fn start(
    arguments: &StartArguments,
    directory: &SessionDirectory,
    ports: &mut ListenPorts,
) -> Result<(), ListenFailure> {
    refuse_open_conversation(directory, ports)?;
    let request = request_of(arguments, ports)?;
    begin_listening(directory, &mut ports.units, &request)?;
    let report = json!({"listening": true, "sources": request.sources});
    ports.output.line(&python_json(&report));
    Ok(())
}
