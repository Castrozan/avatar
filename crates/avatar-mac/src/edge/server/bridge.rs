use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::ws::Message;

use super::state::{PageSender, locked};
use crate::frame::Frame;
use crate::relay::{CallRelay, PageAdmission};

#[derive(Debug, Default)]
pub struct CallBridge {
    relay: Mutex<CallRelay>,
    senders: Mutex<BTreeMap<u64, PageSender>>,
    next: AtomicU64,
}

fn deliver(senders: &BTreeMap<u64, PageSender>, recipients: &[u64], frame: &Frame) {
    let Frame::Binary(bytes) = frame else { return };
    for sender in recipients.iter().filter_map(|id| senders.get(id)) {
        let _ = sender.send(Message::Binary(bytes.clone().into()));
    }
}

impl CallBridge {
    pub fn admit_page(&self) -> PageAdmission {
        locked(&self.relay).admit_page()
    }

    pub fn page_left(&self) {
        locked(&self.relay).page_left();
    }

    pub fn add_capturer(&self, sender: PageSender) -> u64 {
        let capturer = self.next.fetch_add(1, Ordering::SeqCst);
        locked(&self.relay).add_capturer(capturer);
        locked(&self.senders).insert(capturer, sender);
        capturer
    }

    pub fn remove_capturer(&self, capturer: u64) {
        locked(&self.relay).remove_capturer(capturer);
        locked(&self.senders).remove(&capturer);
    }

    pub fn broadcast(&self, frame: &Frame) {
        let recipients = locked(&self.relay).recipients(frame).unwrap_or_default();
        deliver(&locked(&self.senders), &recipients, frame);
    }
}
