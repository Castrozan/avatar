use std::cell::RefCell;
use std::rc::Rc;

use serde_json::Value;

#[derive(Clone, Default)]
pub(crate) struct Log(Rc<RefCell<Vec<Value>>>);

impl Log {
    pub(crate) fn push(&self, entry: Value) {
        self.0.borrow_mut().push(entry);
    }

    pub(crate) fn extend(&self, entries: impl IntoIterator<Item = Value>) {
        self.0.borrow_mut().extend(entries);
    }

    pub(crate) fn entries(&self) -> Vec<Value> {
        self.0.borrow().clone()
    }
}
