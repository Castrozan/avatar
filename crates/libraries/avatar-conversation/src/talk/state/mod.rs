pub mod addressee;
pub mod configuration;
pub mod elsewhere;
pub mod heard;
pub mod phase;
pub mod seat;

pub const CONVERSATION_TARGET: &str = "avatar-conversation.target";
pub const MEETING_TARGET: &str = "avatar.target";
pub const WINDOW_TARGET: &str = "avatar-window.target";

const PANE_VARIABLE: &str = "HERDR_PANE_ID";

#[must_use]
pub fn this_pane(environment: &impl crate::talk::ports::Environment) -> Option<String> {
    environment.variable(PANE_VARIABLE)
}
