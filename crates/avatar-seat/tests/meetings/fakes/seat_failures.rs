use avatar_seat::local::SeatFailure;

use super::failures::edge_type;

pub(crate) fn seat_type(failure: &SeatFailure) -> &'static str {
    match failure {
        SeatFailure::Runtime(_) => "RuntimeError",
        SeatFailure::Timeout(_) => "TimeoutError",
        SeatFailure::Edge(edge) => edge_type(edge),
    }
}
