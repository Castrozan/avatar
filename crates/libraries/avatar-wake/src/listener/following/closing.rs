use avatar_floor::quiet::{ListenerPurpose, QuietBoundaries, Seat};

use super::super::boundaries::Boundaries;
use super::super::records::report_closing;

pub(super) struct QuietView<'a> {
    pub(super) boundaries: Boundaries<'a>,
}

impl QuietBoundaries for QuietView<'_> {
    fn quiet_seconds(&self) -> f64 {
        self.boundaries.seat.quiet_seconds()
    }

    fn listening_purpose(&self) -> ListenerPurpose {
        let purpose = self.boundaries.seat.listening_purpose();
        purpose.ok().flatten().unwrap_or(ListenerPurpose::Live)
    }

    fn open_seat(&self) -> Seat {
        self.boundaries.effects.open_seat().unwrap_or(Seat::Room)
    }

    fn discord_call_ended(&self) -> bool {
        self.boundaries.effects.discord_call_ended()
    }

    fn report(&self, message: &str) {
        report_closing(self.boundaries.effects, message);
    }

    fn close_conversation(&self) {
        self.boundaries.effects.close_conversation();
    }
}
