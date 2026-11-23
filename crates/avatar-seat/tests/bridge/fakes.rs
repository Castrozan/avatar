use std::io;
use std::sync::{Arc, Mutex};

use avatar_seat::bridge::{AudioTools, Capture, Playback};

pub(crate) const VOICE: [u8; 960] = [1; 960];

pub(crate) type Bytes = Arc<Mutex<Vec<u8>>>;

#[derive(Default, Clone)]
pub(crate) struct Heard {
    pub(crate) microphone: Bytes,
    pub(crate) meeting: Bytes,
}

pub(crate) fn copied(bytes: &Mutex<Vec<u8>>) -> Vec<u8> {
    bytes.lock().map_or_else(
        |poisoned| poisoned.into_inner().clone(),
        |held| held.clone(),
    )
}

pub(crate) fn pick<'sink>(
    meeting: bool,
    first: &'sink Bytes,
    second: &'sink Bytes,
) -> &'sink Bytes {
    if meeting { first } else { second }
}

impl Heard {
    pub(crate) fn microphone_bytes(&self) -> Vec<u8> {
        copied(&self.microphone)
    }

    pub(crate) fn meeting_bytes(&self) -> Vec<u8> {
        copied(&self.meeting)
    }

    fn sink_for(&self, command: &[String]) -> Bytes {
        let meeting = command
            .iter()
            .any(|word| word.contains("AvatarRemoteMeeting"));
        Arc::clone(pick(meeting, &self.meeting, &self.microphone))
    }
}

impl FakeTools {
    pub(crate) fn new(heard: Heard, working: bool) -> Self {
        Self { heard, working }
    }
}

#[derive(Clone)]
pub(crate) struct FakeTools {
    pub(crate) heard: Heard,
    pub(crate) working: bool,
}

pub(crate) struct FakePlayback(Bytes);

pub(crate) struct FakeCapture(bool);

pub(crate) fn available(working: bool) -> io::Result<()> {
    working
        .then_some(())
        .ok_or_else(|| io::Error::other("no pacat"))
}

impl AudioTools for FakeTools {
    type Playback = FakePlayback;
    type Capture = FakeCapture;

    async fn playback(&self, command: Vec<String>) -> io::Result<FakePlayback> {
        available(self.working)?;
        Ok(FakePlayback(self.heard.sink_for(&command)))
    }

    async fn capture(&self, _command: Vec<String>) -> io::Result<FakeCapture> {
        Ok(FakeCapture(false))
    }
}

impl Playback for FakePlayback {
    async fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.0
            .lock()
            .map_err(|_| io::Error::other("poisoned"))?
            .extend_from_slice(bytes);
        Ok(())
    }

    async fn end(&mut self) {}
}

pub(crate) async fn voice_once(already_sent: &mut bool) -> io::Result<Vec<u8>> {
    if *already_sent {
        return std::future::pending().await;
    }
    *already_sent = true;
    Ok(VOICE.to_vec())
}

impl Capture for FakeCapture {
    async fn read(&mut self, _limit: usize) -> io::Result<Vec<u8>> {
        voice_once(&mut self.0).await
    }

    async fn end(&mut self) {}
}
