use std::io;

use avatar_floor::quiet::Seat;

const LIP_SYNC_SINK: &str = "AvatarLipSync";

#[derive(Clone, Copy)]
pub(crate) struct Sink {
    pub(crate) device: Option<&'static str>,
    pub(crate) client: &'static str,
}

const SPEAKERS: Sink = Sink {
    device: None,
    client: "avatar-live",
};
const LIP_SYNC: Sink = Sink {
    device: Some(LIP_SYNC_SINK),
    client: "avatar-live-lip-sync",
};

pub(crate) fn sinks_of(seat: Seat) -> io::Result<Vec<Sink>> {
    match seat {
        Seat::Room => Ok(vec![SPEAKERS, LIP_SYNC]),
        Seat::Remote => Ok(vec![LIP_SYNC]),
        Seat::Discord => Err(io::Error::other("The live voice has no Discord output")),
    }
}
