use avatar_floor::quiet::Seat;
use avatar_routing::names::session::Session;
use avatar_wake::addressing::DecisionUnavailable;
use avatar_wake::listener::{Boundary, WakeEffects};
use serde_json::{Value, json};

use super::outcome::{failure_of, outcome, scrubbed, seat_of, sessions_of, summary};
use super::world::World;

fn as_text(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_owned()
}

fn decision_of(response: &Value) -> Result<Value, DecisionUnavailable> {
    match response["error"].as_bool() {
        Some(true) => Err(DecisionUnavailable {
            reason: response["message"].as_str().unwrap_or_default().to_owned(),
        }),
        _ => Ok(response["answers"].clone()),
    }
}

impl WakeEffects for World {
    fn sessions(&self) -> Vec<Session> {
        sessions_of(&self.answer("sessions"))
    }

    fn discord_call_host(&self) -> Option<String> {
        self.text("discord_call_host")
    }

    fn open_seat(&self) -> Option<Seat> {
        seat_of(&self.answer("open_seat"))
    }

    fn discord_call_ended(&self) -> bool {
        self.flag("discord_call_ended")
    }

    fn transcribe(&self, segment: &[u8]) -> Boundary<String> {
        self.record("transcribe", vec![json!(segment.len())]);
        let response = self.script.next("transcribe");
        outcome(&response, as_text)
    }

    fn decide(&self, _input: &Value, _questions: &Value) -> Result<Value, DecisionUnavailable> {
        decision_of(&self.answer("decide"))
    }

    fn room_that_heard(&self, seconds: f64) -> Option<String> {
        self.record("room_that_heard", vec![json!(seconds)]);
        self.script
            .next("room_that_heard")
            .as_str()
            .map(str::to_owned)
    }

    fn claim_wake(&self, session: &Session) -> bool {
        self.record("claim_wake", vec![json!(session.pane), json!(session.host)]);
        self.script.next("claim_wake").as_bool().unwrap_or_default()
    }

    fn prompt_now(&self, pane: &str, text: &str, host: Option<&str>) -> Boundary<bool> {
        self.record("prompt_now", vec![json!(pane), summary(text), json!(host)]);
        let response = self.script.next("prompt_now");
        let delivered = outcome(&response, |value| value.as_bool().unwrap_or_default());
        let cost = self.number("prompt_cost");
        delivered.inspect(|_| self.clock.set(self.clock.get() + cost))
    }

    fn notify_at_idle(&self, pane: &str, text: &str, host: Option<&str>) -> Boundary<()> {
        self.record(
            "notify_at_idle",
            vec![json!(pane), summary(text), json!(host)],
        );
        failure_of(&self.script.next("notify_at_idle")).map_or(Ok(()), Err)
    }

    fn close_conversation(&self) {
        self.record("close_conversation", Vec::new());
    }

    fn report(&self, line: &str) {
        self.record("report", vec![scrubbed(line)]);
    }

    fn room_name(&self) -> Option<String> {
        self.config["room"].as_str().map(str::to_owned)
    }

    fn timestamp(&self) -> String {
        "T".to_owned()
    }

    fn now(&self) -> f64 {
        self.record("now", Vec::new());
        self.clock.get()
    }

    fn monotonic(&self) -> f64 {
        self.record("monotonic", Vec::new());
        self.clock.get()
    }

    fn sleep(&self, seconds: f64) {
        self.record("sleep", vec![json!(format!("{seconds}"))]);
        self.clock.set(self.clock.get() + seconds);
        self.sleeps.set(self.sleeps.get() + 1);
        let limit = self.config["stop_after_sleeps"]
            .as_u64()
            .unwrap_or(u64::MAX);
        self.stopped
            .set(self.stopped.get() || self.sleeps.get() >= limit);
    }

    fn pause(&self, seconds: f64) {
        self.record("pause", vec![json!(format!("{seconds}"))]);
    }
}
