use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::ports::{
    ActivityLog, AgentPane, ConsumerLock, Floor, ListenerSession, ListeningRequest, Readiness,
    Units,
};
use avatar_listening::conversation_addressee::Addressee;
use avatar_listening::conversation_state::ListenerPurpose;
use serde_json::{Map, Value, json};

use super::World;

const BUSY_MESSAGE: &str = "Another avatar-talk hear, ask or follow is already listening";

fn purpose_name(purpose: ListenerPurpose) -> &'static str {
    match purpose {
        ListenerPurpose::Conversation => "conversation",
        ListenerPurpose::Meeting => "meeting",
        ListenerPurpose::Live => "live",
    }
}

fn check_names(checks: &[Readiness]) -> Vec<&'static str> {
    checks.iter().map(|check| check.name()).collect()
}

fn acquire_consumer(world: &mut World) -> Result<(), TalkFailure> {
    let busy = world.scenario_field("consumer_busy") == &Value::Bool(true);
    if !busy {
        world.stamp(json!({"call": "consumer", "event": "acquire"}));
    }
    busy.then(|| TalkFailure::new(BUSY_MESSAGE))
        .map_or(Ok(()), Err)
}

impl Units for World {
    fn unit_active(&mut self, unit: &str) -> bool {
        let series = self.scenario_field("units")[unit].clone();
        self.timeline(&series, &Value::Bool(false)) == Value::Bool(true)
    }

    fn bring_up(&mut self, target: &str, checks: &[Readiness]) -> Result<(), TalkFailure> {
        self.stamp(json!({"call": "bring_up", "target": target, "checks": check_names(checks)}));
        self.scripted("calls", "bring_up").map(|_| ())
    }

    fn wait_until_ready(
        &mut self,
        units: &[&str],
        checks: &[Readiness],
    ) -> Result<(), TalkFailure> {
        self.stamp(
            json!({"call": "wait_until_ready", "units": units, "checks": check_names(checks)}),
        );
        self.scripted("calls", "wait_until_ready").map(|_| ())
    }

    fn remote_seat_listening(&mut self) -> bool {
        let series = self.scenario_field("remote_seat_listening").clone();
        self.timeline(&series, &Value::Bool(false)) == Value::Bool(true)
    }
}

impl ListenerSession for World {
    fn begin_listening(&mut self, request: &ListeningRequest) -> Result<(), TalkFailure> {
        self.stamp(json!({"call": "begin_listening", "purpose": purpose_name(request.purpose),
            "sources": {"human": request.human_source}, "settings": Value::Object(request.settings.clone())}));
        self.scripted("calls", "begin_listening").map(|_| ())
    }

    fn erase_session_records(&mut self) -> Result<(), TalkFailure> {
        self.stamp(json!({"call": "erase_session_records"}));
        self.scripted("calls", "erase_session_records").map(|_| ())
    }
}

impl Floor for World {
    fn give_the_room_to(&mut self, addressee: &Addressee) -> Result<(), TalkFailure> {
        self.stamp(
            json!({"call": "give_the_room_to", "pane": addressee.pane, "servant": addressee.servant,
            "heard_through": addressee.heard_through, "host": addressee.host}),
        );
        self.scripted("calls", "give_the_room_to").map(|_| ())
    }

    fn hand_to_remote_agent(&mut self, agent: &AgentPane) -> Result<bool, TalkFailure> {
        self.stamp(json!({"call": "hand_to_remote_agent", "host": agent.host, "pane": agent.pane}));
        self.scripted("calls", "hand_to_remote_agent").map(|_| true)
    }
}

impl ActivityLog for World {
    fn report_talk(&mut self, action: &str, pane: Option<&str>) {
        self.stamp(json!({"call": "report_talk", "action": action, "pane": pane}));
    }

    fn report_speech(&mut self, pane: Option<&str>, details: &Map<String, Value>) {
        self.stamp(json!({"call": "report_speech", "pane": pane, "details": details}));
    }
}

impl ConsumerLock for World {
    fn acquire(&mut self) -> Result<(), TalkFailure> {
        acquire_consumer(self)
    }

    fn release(&mut self) {
        self.stamp(json!({"call": "consumer", "event": "release"}));
    }
}
