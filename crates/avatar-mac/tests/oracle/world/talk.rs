use avatar_listening::conversation_state::ConversationCommand;
use avatar_mac::failure::Failure;
use avatar_mac::talk::{Reports, RoomCommands, SeatProbe, TalkRequest};
use serde_json::{Value, json};

use super::World;

const COMMANDS: [(&str, ConversationCommand); 10] = [
    ("open", ConversationCommand::Open),
    ("open-live", ConversationCommand::OpenLive),
    ("close", ConversationCommand::Close),
    ("ask", ConversationCommand::Ask),
    ("hear", ConversationCommand::Hear),
    ("follow", ConversationCommand::Follow),
    ("say", ConversationCommand::Say),
    ("show", ConversationCommand::Show),
    ("hide", ConversationCommand::Hide),
    ("status", ConversationCommand::Status),
];

fn command_of(name: &str) -> ConversationCommand {
    let known = COMMANDS.iter().find(|entry| entry.0 == name);
    known.map_or(ConversationCommand::Status, |entry| entry.1)
}

pub(super) fn text(value: &Value) -> Option<String> {
    value.as_str().map(str::to_owned)
}

fn seat_flag(world: &mut World, event: Value, name: &str) -> bool {
    world.log.push(event);
    world.config["seat"][name] == true
}

impl SeatProbe for World {
    fn seat_reachable(&mut self) -> bool {
        seat_flag(self, json!(["seat_health"]), "reachable")
    }

    fn conversation_listening(&mut self) -> bool {
        seat_flag(self, json!(["conversation_listening"]), "listening")
    }

    fn meeting_held(&mut self) -> bool {
        seat_flag(self, json!(["meeting_held"]), "meeting_held")
    }

    fn meeting_seat_kept_here(&mut self) -> Result<bool, Failure> {
        let kept = seat_flag(self, json!(["meeting_kept"]), "kept_here");
        self.fails("meeting_kept").map(|()| kept)
    }

    fn leave_seat(&mut self) -> Result<(), Failure> {
        self.log.push(json!(["leave_seat"]));
        self.fails("leave_seat")
    }

    fn seat_did_not_attach_message(&self) -> String {
        "The seat page did not attach; check that Google Chrome for Testing may use the microphone"
            .to_owned()
    }
}

impl Reports for World {
    fn print_status(&mut self, is_open: bool, live: bool, seat: Option<&str>) {
        self.log.push(json!(["print_status", is_open, live, seat]));
    }

    fn print_room_held_by(&mut self, servant: &str) {
        self.log.push(json!(["print_room_held_by", servant]));
    }

    fn print_closed(&mut self) {
        self.log.push(json!(["print_closed"]));
    }
}

impl RoomCommands for World {
    fn parse_request(&mut self, _arguments: &[String]) -> Result<TalkRequest, Failure> {
        let request = &self.config["request"];
        Ok(TalkRequest {
            command: command_of(request["command"].as_str().unwrap_or_default()),
            language: text(&request["language"]),
            character: text(&request["character"]),
            text: text(&request["text"]).unwrap_or_default(),
            emotion: text(&request["emotion"]).unwrap_or_default(),
            voice: text(&request["voice"]),
            wait: request["wait"].as_f64().unwrap_or(0.0),
        })
    }

    fn hear(&mut self, _request: &TalkRequest) -> Result<(), Failure> {
        self.log.push(json!(["hear"]));
        Ok(())
    }

    fn follow(&mut self, _request: &TalkRequest) -> Result<(), Failure> {
        self.log.push(json!(["follow"]));
        Ok(())
    }

    fn holder_other_than(&mut self, pane: Option<&str>) -> Option<String> {
        self.log.push(json!(["holder_other_than", pane]));
        text(&self.config["holder"])
    }
}
