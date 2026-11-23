use avatar_conversation::talk::surfaces::seat_preparation::prepare;
use avatar_floor::quiet::Seat;
use avatar_seat::local::SeatFailure;
use avatar_seat::meetings::start::SeatPreparation;

use crate::support::runtime::session_directory;
use crate::talk::TalkBoundaries;

pub(super) struct ConversationSeatPreparation;

impl SeatPreparation for ConversationSeatPreparation {
    fn prepare(&self, seat: Seat) -> Result<(), SeatFailure> {
        let mut boundaries = TalkBoundaries::on_this_machine(session_directory());
        prepare(&mut boundaries, seat).map_err(|failure| SeatFailure::Runtime(failure.to_string()))
    }
}
