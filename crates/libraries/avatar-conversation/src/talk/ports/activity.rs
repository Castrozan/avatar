use serde_json::{Map, Value};

use crate::talk::TalkFailure;

pub trait ActivityLog {
    fn report_talk(&mut self, action: &str, pane: Option<&str>);
    fn report_speech(&mut self, pane: Option<&str>, details: &Map<String, Value>);
}

pub trait ConsumerLock {
    fn acquire(&mut self) -> Result<(), TalkFailure>;
    fn release(&mut self);
}
