use std::collections::VecDeque;
use std::io;

use avatar_listening::listener::capture::{CaptureLauncher, CaptureStream, NoBargeIn};
use avatar_listening::listener::daemon::ListenerKit;

pub(super) struct Frames(VecDeque<Vec<u8>>);

impl CaptureStream for Frames {
    fn read_frame(&mut self) -> Option<Vec<u8>> {
        self.0.pop_front()
    }

    fn end(&mut self) {}
}

pub(super) struct OneShot(Option<Vec<Vec<u8>>>);

impl CaptureLauncher for OneShot {
    type Stream = Frames;

    fn launch(&mut self, _device: &str) -> io::Result<Frames> {
        let frames = self.0.take().ok_or(io::ErrorKind::NotFound)?;
        Ok(Frames(frames.into()))
    }
}

pub(super) struct Kit(pub(super) Vec<Vec<u8>>);

fn every_pause_ends_the_turn(_audio: &[u8]) -> bool {
    true
}

impl ListenerKit for Kit {
    type Launcher = OneShot;
    type Judge = fn(&[u8]) -> bool;
    type Hearing = NoBargeIn;

    fn launcher(&self) -> OneShot {
        OneShot(Some(self.0.clone()))
    }

    fn judge(&self) -> Self::Judge {
        every_pause_ends_the_turn
    }

    fn hearing(&self) -> NoBargeIn {
        NoBargeIn
    }
}
