use std::io::{self, Write};
use std::sync::{Mutex, MutexGuard, PoisonError};

use axum::extract::ws::Message;
use tokio::sync::{mpsc, watch};

use super::bridge::CallBridge;
use super::files::PageFiles;
use crate::seat::{Hush, SpeechTicket};

pub(super) type PageSender = mpsc::UnboundedSender<Message>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Presence {
    pub attached: bool,
    pub detached: bool,
}

pub(super) fn locked<Value>(mutex: &Mutex<Value>) -> MutexGuard<'_, Value> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

pub struct SeatState {
    output: Mutex<Box<dyn Write + Send>>,
    page: Mutex<Option<PageSender>>,
    presence: watch::Sender<Presence>,
    hush: Mutex<Hush>,
    failure: Mutex<Option<String>>,
    pub call: CallBridge,
    pub files: PageFiles,
    pub origins: Vec<String>,
}

impl SeatState {
    #[must_use]
    pub fn new(output: Box<dyn Write + Send>, files: PageFiles, origins: Vec<String>) -> Self {
        Self {
            output: Mutex::new(output),
            page: Mutex::new(None),
            presence: watch::channel(Presence::default()).0,
            hush: Mutex::new(Hush::default()),
            failure: Mutex::new(None),
            call: CallBridge::default(),
            files,
            origins,
        }
    }

    #[must_use]
    pub fn subscribe(&self) -> watch::Receiver<Presence> {
        self.presence.subscribe()
    }

    pub fn attach(&self, page: PageSender) -> bool {
        let first = !self.presence.borrow().attached;
        first.then(|| self.seat_page(page)).is_some()
    }

    fn seat_page(&self, page: PageSender) {
        *locked(&self.page) = Some(page);
        self.presence
            .send_modify(|presence| presence.attached = true);
    }

    pub fn page_left(&self) {
        *locked(&self.page) = None;
        self.presence
            .send_modify(|presence| presence.detached = true);
    }

    #[must_use]
    pub fn attached(&self) -> bool {
        self.presence.borrow().attached
    }

    #[must_use]
    pub fn page_present(&self) -> bool {
        locked(&self.page).is_some()
    }

    pub fn send_to_page(&self, message: Message) {
        let _ = locked(&self.page)
            .as_ref()
            .map(|sender| sender.send(message));
    }

    pub fn write_output(&self, bytes: &[u8]) -> io::Result<()> {
        let mut output = locked(&self.output);
        output.write_all(bytes)?;
        output.flush()
    }

    pub fn hush(&self) {
        let notice = locked(&self.hush).hush(self.page_present());
        let _ = notice.map(|text| self.send_to_page(Message::text(text)));
    }

    pub fn begin_speech(&self) -> SpeechTicket {
        locked(&self.hush).begin_speech()
    }

    #[must_use]
    pub fn failure(&self) -> Option<String> {
        locked(&self.failure).clone()
    }

    pub fn record_failure(&self, text: String) {
        *locked(&self.failure) = Some(text);
    }
}
