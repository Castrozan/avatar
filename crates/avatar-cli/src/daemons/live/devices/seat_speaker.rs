use std::io;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use avatar_live::boundaries::Speaker;
use avatar_mac::client::{HUSH_TIMEOUT_SECONDS, PORT};
use avatar_mac::edge::client::{SeatClient, hush_the_seat};
use avatar_mac::edge::room_audio::{
    CONTROL_URL, ControlLevels, SeatSpeaker, SeatStreamSink, SpeakerParts,
};

use crate::daemons::live::shutdown::Shutdown;
use crate::mac::environment::seat_config;

type Inner = SeatSpeaker<ControlLevels>;

pub(crate) struct MacSpeaker {
    inner: Option<Inner>,
    shutdown: Arc<Shutdown>,
}

fn off_runtime<Output: Send>(work: impl FnOnce() -> Output + Send) -> Option<Output> {
    thread::scope(|scope| scope.spawn(work).join().ok())
}

fn hush() {
    hush_the_seat(PORT, Duration::from_secs(HUSH_TIMEOUT_SECONDS));
}

fn inner_of(rate: u32) -> io::Result<Inner> {
    let browser = seat_config()?.browser_name();
    let message =
        format!("The seat page did not attach; check that {browser} may use the microphone");
    let client = SeatClient::on_this_mac().map_err(|failure| io::Error::other(failure.message))?;
    Ok(SeatSpeaker::start(SpeakerParts {
        sink: Box::new(SeatStreamSink::new(client, rate, &message)),
        hush: Box::new(hush),
        voice: ControlLevels::connect(CONTROL_URL)?,
        rate,
    }))
}

fn report(shutdown: &Shutdown, failure: Option<String>) {
    if let Some(message) = failure {
        shutdown.fail(message);
    }
}

fn close_all(inner: Option<Inner>) {
    if let Some(mut speaker) = inner {
        speaker.close();
    }
}

impl MacSpeaker {
    pub(crate) fn start(rate: u32, shutdown: Arc<Shutdown>) -> io::Result<Self> {
        let inner = off_runtime(move || inner_of(rate))
            .ok_or_else(|| io::Error::other("The seat speaker failed to start"))??;
        Ok(Self {
            inner: Some(inner),
            shutdown,
        })
    }

    fn report_failure(&self) {
        let failure = self.inner.as_ref().and_then(Inner::failure);
        report(&self.shutdown, failure);
    }
}

impl Speaker for MacSpeaker {
    fn play(&mut self, pcm: &[u8]) {
        self.report_failure();
        self.inner.iter_mut().for_each(|inner| inner.play(pcm));
    }

    fn buffered_seconds(&self) -> f64 {
        self.inner.as_ref().map_or(0.0, Inner::buffered_seconds)
    }

    fn clear(&mut self) {
        let inner = self.inner.as_mut();
        off_runtime(move || inner.into_iter().for_each(Inner::clear));
    }
}

impl Drop for MacSpeaker {
    fn drop(&mut self) {
        let inner = self.inner.take();
        off_runtime(move || close_all(inner));
    }
}
