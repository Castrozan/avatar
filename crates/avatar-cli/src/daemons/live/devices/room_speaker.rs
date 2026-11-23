use std::io;
use std::sync::Arc;

use avatar_floor::quiet::Seat;
use avatar_live::boundaries::Speaker;

use super::seat_speaker::MacSpeaker;
use super::sinks::sinks_of;
use super::speaker::PacatSpeaker;
use crate::daemons::live::shutdown::Shutdown;

pub(crate) enum RoomSpeaker {
    Pacat(PacatSpeaker),
    Mac(MacSpeaker),
}

fn start_pacat(rate: u32, seat: Seat, shutdown: Arc<Shutdown>) -> io::Result<RoomSpeaker> {
    PacatSpeaker::start(rate, &sinks_of(seat)?, shutdown).map(RoomSpeaker::Pacat)
}

fn start_for_platform(rate: u32, seat: Seat, shutdown: Arc<Shutdown>) -> io::Result<RoomSpeaker> {
    if cfg!(target_os = "macos") {
        MacSpeaker::start(rate, shutdown).map(RoomSpeaker::Mac)
    } else {
        start_pacat(rate, seat, shutdown)
    }
}

impl RoomSpeaker {
    pub(crate) fn start(rate: u32, seat: Seat, shutdown: Arc<Shutdown>) -> io::Result<Self> {
        start_for_platform(rate, seat, shutdown)
    }
}

impl Speaker for RoomSpeaker {
    fn play(&mut self, pcm: &[u8]) {
        match self {
            Self::Pacat(speaker) => speaker.play(pcm),
            Self::Mac(speaker) => speaker.play(pcm),
        }
    }

    fn buffered_seconds(&self) -> f64 {
        match self {
            Self::Pacat(speaker) => speaker.buffered_seconds(),
            Self::Mac(speaker) => speaker.buffered_seconds(),
        }
    }

    fn clear(&mut self) {
        match self {
            Self::Pacat(speaker) => speaker.clear(),
            Self::Mac(speaker) => speaker.clear(),
        }
    }
}
