use avatar_mac::failure::Failure;
use avatar_mac::stage::StageSurface;
use serde_json::json;

use super::{World, failure_of};

impl StageSurface for World {
    fn answers(&mut self, port: u16) -> bool {
        self.event(json!(["answers", port]));
        let rule = self.config["answers"][port.to_string()]
            .as_str()
            .unwrap_or("always")
            .to_owned();
        self.scripted(&format!("answers{port}"), &rule)
    }

    fn hide_window_surface(&mut self) -> Result<(), Failure> {
        self.event(json!(["hide"]));
        self.fails("hide")
    }

    fn open_window(&mut self, page: &str, port: u16) -> Result<(), Failure> {
        self.event(json!(["open_window", page, port]));
        self.fails("open_window")
    }

    fn avatar_command(
        &mut self,
        arguments: &[String],
        timeout_seconds: f64,
    ) -> Result<(), Failure> {
        self.commands += 1;
        self.event(json!(["command", arguments, timeout_seconds]));
        let first = self.config["command_fail_first"].as_u64().unwrap_or(0);
        let kind = self.config["command_fail_kind"]
            .as_str()
            .unwrap_or("called")
            .to_owned();
        (self.commands <= first)
            .then(|| failure_of(&kind, "command"))
            .map_or(Ok(()), Err)
    }
}
