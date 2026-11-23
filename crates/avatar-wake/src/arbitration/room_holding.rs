use avatar_routing::names::session::Session;

pub const HEARING_SLACK_SECONDS: f64 = 3.0;
const SAMPLE_RATE: f64 = 16000.0;
const ANOTHER_ROOM: &str = "another room";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeardSegment {
    pub audio_byte_count: u32,
    pub heard_at: f64,
}

impl HeardSegment {
    #[must_use]
    pub fn began_seconds_ago(&self, now: f64) -> f64 {
        let audio_seconds = f64::from(self.audio_byte_count) / (2.0 * SAMPLE_RATE);
        audio_seconds + now - self.heard_at
    }
}

pub trait HoldingBoundaries {
    fn discord_call_host(&self) -> Option<String>;
    fn room_that_heard(&self, seconds: f64) -> Option<String>;
    fn now(&self) -> f64;
    fn claim_wake(&self, session: &Session) -> bool;
    fn report(&self, text: &str);
}

#[must_use]
pub fn reason_to_leave(
    boundaries: &dyn HoldingBoundaries,
    dry_run: bool,
    session: &Session,
    heard: &HeardSegment,
) -> Option<String> {
    discord_reason(boundaries)
        .or_else(|| room_reason(boundaries, heard))
        .or_else(|| wake_reason(boundaries, dry_run, session))
}

fn discord_reason(boundaries: &dyn HoldingBoundaries) -> Option<String> {
    boundaries
        .discord_call_host()
        .filter(|host| !host.is_empty())
        .map(|host| format!("the Discord call on {host}"))
}

fn room_reason(boundaries: &dyn HoldingBoundaries, heard: &HeardSegment) -> Option<String> {
    let seconds = heard.began_seconds_ago(boundaries.now()) + HEARING_SLACK_SECONDS;
    boundaries
        .room_that_heard(seconds)
        .filter(|room| !room.is_empty())
        .map(|room| format!("the conversation on {room}"))
}

fn wake_reason(
    boundaries: &dyn HoldingBoundaries,
    dry_run: bool,
    session: &Session,
) -> Option<String> {
    (!dry_run && !boundaries.claim_wake(session)).then(|| ANOTHER_ROOM.to_owned())
}

#[must_use]
pub fn holds_the_room(
    boundaries: &dyn HoldingBoundaries,
    dry_run: bool,
    session: &Session,
    heard: &HeardSegment,
) -> bool {
    let reason = reason_to_leave(boundaries, dry_run, session, heard);
    if let Some(reason) = &reason {
        boundaries.report(&format!("left {} to {reason}", session.servant));
    }
    reason.is_none()
}
