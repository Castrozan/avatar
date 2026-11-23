use avatar_listening::conversation_state::ConversationPhase;
use avatar_mac::failure::Failure;
use avatar_mac::meeting::{MeetingCamera, SeatHealth};
use avatar_mac::session::{Console, ListeningPlan, RoomSession, SeatControl};
use serde_json::{Value, json};

use super::World;
use super::listening::{phase_of, purpose_name, settings_of, sources_of};

fn health_of(entry: &Value) -> Option<SeatHealth> {
    let fields = entry.as_object()?;
    Some(SeatHealth {
        failure: fields
            .get("failure")
            .and_then(Value::as_str)
            .map(str::to_owned),
        attached: fields.get("attached").is_some_and(|flag| flag == true),
    })
}

fn scripted_entry(world: &mut World) -> Option<Value> {
    let calls = world.calls.entry("health".to_owned()).or_default();
    *calls += 1;
    let scripted = world.config["health"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let at = usize::try_from(*calls - 1).unwrap_or(0);
    scripted.get(at).or(scripted.last()).cloned()
}

pub(crate) fn scripted_health(world: &mut World) -> Option<SeatHealth> {
    let entry = scripted_entry(world)?;
    world.log.push(json!(["health"]));
    health_of(&entry)
}

impl RoomSession for World {
    fn phase(&mut self) -> ConversationPhase {
        phase_of(self.config["phase"].as_str().unwrap_or_default())
    }

    fn begin_listening(&mut self, plan: &ListeningPlan) -> Result<(), Failure> {
        let purpose = purpose_name(plan.purpose);
        self.log.push(json!([
            "begin_listening",
            purpose,
            sources_of(plan),
            settings_of(plan)
        ]));
        self.fails("begin_listening")
    }

    fn stop_listening(&mut self) -> Result<(), Failure> {
        self.log.push(json!(["stop_listening"]));
        self.fails("stop_listening")
    }

    fn erase_session_records(&mut self) -> Result<(), Failure> {
        self.log.push(json!(["erase_session_records"]));
        self.fails("erase_session_records")
    }
}

fn attached_by(world: &mut World) -> bool {
    match world.config["attached"].as_str().map(str::to_owned) {
        Some(rule) => world.scripted("attached", &rule),
        None => world.config["seat_attached"] == true,
    }
}

impl SeatControl for World {
    fn seat_attached(&mut self) -> bool {
        self.log.push(json!(["seat_attached"]));
        attached_by(self)
    }

    fn hush(&mut self) {
        self.log.push(json!(["hush"]));
    }

    fn close_browser(&mut self) -> Result<(), Failure> {
        self.log.push(json!(["close_browser"]));
        self.fails("close_browser")
    }
}

impl MeetingCamera for World {
    fn start_camera(&mut self) -> Result<(), Failure> {
        self.log.push(json!(["camera_start"]));
        self.fails("camera_start")
    }

    fn stop_camera(&mut self) -> Result<(), Failure> {
        self.log.push(json!(["camera_stop"]));
        self.fails("camera_stop")
    }

    fn camera_running(&mut self) -> bool {
        self.log.push(json!(["camera_running"]));
        self.config["camera_running"] == true
    }
}

impl Console for World {
    fn print_line(&mut self, line: &str) {
        self.stdout.push_str(line);
        self.stdout.push('\n');
    }

    fn print_error(&mut self, line: &str) {
        self.log.push(json!(["stderr", line]));
    }
}
