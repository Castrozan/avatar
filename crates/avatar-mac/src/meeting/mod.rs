mod ending;
mod ports;
mod steps;
mod usage;

pub use ending::end;
pub use ports::{CallDefaults, MeetingCamera, MeetingRoom, SeatHealth};
pub use steps::{controls, listen_to_the_call, start, stop};
pub use usage::{ListenCommand, carry_out_verb, listen_command};

pub const CONTROLS_PAGE: &str = "controls.html";
pub const CALL_DEVICES_READY: &str = "Avatar ready. This Mac's default microphone is now BlackHole 2ch and its default speaker BlackHole 16ch until avatar-stop; in the call, keep the microphone and speaker on the system default and select the camera OBS Virtual Camera";
pub const CONVERSATION_OPEN: &str =
    "A conversation is open on this Mac; run avatar-talk close before using the meeting commands";
pub const MEETING_OPEN: &str =
    "A meeting is running on this Mac; run avatar-stop to end it before talking";
pub const CONTROLS_NEED_A_MEETING: &str = "Run avatar-start on this Mac before opening its controls: they choose the camera of the call picture, which is not running here";
pub const USAGE: &str = "Usage: avatar-start | avatar-stop | avatar-controls";
