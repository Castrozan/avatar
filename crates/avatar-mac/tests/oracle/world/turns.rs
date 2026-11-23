use avatar_mac::failure::Failure;
use avatar_mac::talk::TalkRequest;
use avatar_mac::turns::RoomTurns;
use serde_json::json;

use super::World;
use super::talk::text;

impl RoomTurns for World {
    type Command = String;
    type Completed = String;

    fn this_pane(&mut self) -> Option<String> {
        text(&self.config["pane"])
    }

    fn room_addresses(&mut self, pane: Option<&str>) -> bool {
        self.log.push(json!(["addresses", pane]));
        self.config["addresses"] == true
    }

    fn mark_spoken(&mut self) -> Result<(), Failure> {
        self.log.push(json!(["mark_spoken"]));
        self.fails("mark_spoken")
    }

    fn speech_command_on_seat(&mut self, request: &TalkRequest) -> String {
        self.log
            .push(json!(["speech_command", request.text, "room"]));
        "cmd".to_owned()
    }

    fn spoken_in_turn(&mut self, command: &String) -> Result<String, Failure> {
        self.log.push(json!(["spoken_in_turn", command]));
        self.fails("spoken_in_turn")
            .map(|()| "completed".to_owned())
    }

    fn print_and_exit(&mut self, completed: String) -> i32 {
        self.log.push(json!(["print_and_exit", completed]));
        7
    }

    fn listen_after(&mut self, completed: &String, wait: f64) -> Result<(), Failure> {
        self.log.push(json!(["listen_after", completed, wait]));
        Ok(())
    }

    fn acquire_consumer(&mut self) -> Result<(), Failure> {
        self.log.push(json!(["consumer"]));
        self.fails("consumer")
    }

    fn release_consumer(&mut self) {
        self.log.push(json!(["consumer_release"]));
    }
}
