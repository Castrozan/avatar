mod hold;
mod meeting_seat;

pub use hold::{HoldEdges, SeatKeeper, TunnelProcess, hold_seat};
pub use meeting_seat::{CameraFailure, ConversationSeat, MeetingCamera, MeetingSeat, Reporter};

pub const ROUND_SECONDS: u64 = 10;
pub const LOOKS_WITHOUT_A_MEETING_BEFORE_IT_ENDED: u32 = 6;
