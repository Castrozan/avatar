mod answer;
mod decoding;
mod request;
mod routes;
mod seat_page;

pub use answer::{PageAnswer, PageOutcome};
pub use request::{PageFiles, PageRequest, PageState};
pub use routes::{PageRoute, seat_page_routes};
pub use seat_page::SeatPage;

pub const HEALTH_PATH: &str = "/health";
pub const FAILURE_PATH: &str = "/failure";
pub const LEVELS_PATH: &str = "/levels";
