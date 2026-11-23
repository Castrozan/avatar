use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub const HUSH_MESSAGE: &str = "hush";

#[derive(Debug, Clone)]
pub struct SpeechTicket(Arc<AtomicBool>);

impl SpeechTicket {
    #[must_use]
    pub fn is_hushed(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

#[derive(Debug)]
pub struct Hush {
    current: SpeechTicket,
}

impl Default for Hush {
    fn default() -> Self {
        Self {
            current: SpeechTicket(Arc::default()),
        }
    }
}

impl Hush {
    #[must_use]
    pub fn hush(&self, page_present: bool) -> Option<&'static str> {
        self.current.0.store(true, Ordering::SeqCst);
        page_present.then_some(HUSH_MESSAGE)
    }

    pub fn begin_speech(&mut self) -> SpeechTicket {
        self.current.0.store(true, Ordering::SeqCst);
        self.current = SpeechTicket(Arc::default());
        self.current.clone()
    }

    #[must_use]
    pub fn is_hushed(&self) -> bool {
        self.current.is_hushed()
    }
}
