mod http;
mod lifecycle;
mod router;
mod serve;
pub(crate) mod socket;

pub use lifecycle::stay_while_taken;
pub use router::router;
pub use serve::serve_remote_seat;
