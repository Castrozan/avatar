mod bridge;
mod carry;
mod entry;
mod files;
mod page;
mod routes;
mod run;
mod sockets;
mod state;

pub use bridge::CallBridge;
pub use entry::{ServerProgram, hold_seat, relay_microphone};
pub use files::PageFiles;
pub use routes::{microphone_router, seat_router};
pub use run::serve_until_the_page_leaves;
pub use state::{Presence, SeatState};

pub const MICROPHONE_PORT: u16 = 8771;
pub const MICROPHONE_ATTACH_SECONDS: u64 = 30;
pub const PAGE_HOLDS_MICROPHONE: &str = "Another page holds the microphone";

#[must_use]
pub fn allowed_origins(port: u16) -> Vec<String> {
    vec![
        format!("http://localhost:{port}"),
        format!("http://127.0.0.1:{port}"),
    ]
}
