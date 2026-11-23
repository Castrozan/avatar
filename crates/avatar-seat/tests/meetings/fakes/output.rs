use std::cell::RefCell;

use avatar_seat::meetings::Output;

#[derive(Default)]
pub(crate) struct RecordedOutput(RefCell<String>);

impl RecordedOutput {
    pub(crate) fn text(&self) -> String {
        self.0.borrow().clone()
    }
}

impl Output for RecordedOutput {
    fn line(&self, text: &str) {
        let mut recorded = self.0.borrow_mut();
        recorded.push_str(text);
        recorded.push('\n');
    }
}
