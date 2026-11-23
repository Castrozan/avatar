mod boundaries;
mod remote;
mod room;
mod session;
mod speaking;

pub use boundaries::{FloorBoundaries, ServantCharacter};
pub use remote::{RemoteAgent, hand_to_remote_agent};
pub use room::{addressee_of, give_the_room_to};
pub use session::Session;
pub use speaking::{
    claim, claim_among_live_sessions, first_claim, holder_has_a_live_session, may_speak,
};
