use std::collections::BTreeSet;

use crate::frame::Frame;

pub const PAGE_TAKEN_CODE: u16 = 1013;
pub const PAGE_TAKEN_REASON: &str = "Another page carries the call";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageAdmission {
    Carry,
    Refuse { code: u16, reason: &'static str },
}

fn admission_for(already_carrying: bool) -> PageAdmission {
    if already_carrying {
        return PageAdmission::Refuse {
            code: PAGE_TAKEN_CODE,
            reason: PAGE_TAKEN_REASON,
        };
    }
    PageAdmission::Carry
}

#[derive(Debug, Default)]
pub struct CallRelay {
    capturers: BTreeSet<u64>,
    carrying: bool,
}

impl CallRelay {
    pub fn admit_page(&mut self) -> PageAdmission {
        admission_for(std::mem::replace(&mut self.carrying, true))
    }

    pub fn page_left(&mut self) {
        self.carrying = false;
    }

    pub fn add_capturer(&mut self, capturer: u64) {
        self.capturers.insert(capturer);
    }

    pub fn remove_capturer(&mut self, capturer: u64) {
        self.capturers.remove(&capturer);
    }

    #[must_use]
    pub fn is_carrying(&self) -> bool {
        self.carrying
    }

    #[must_use]
    pub fn capturer_ids(&self) -> Vec<u64> {
        self.capturers.iter().copied().collect()
    }

    #[must_use]
    pub fn recipients(&self, frame: &Frame) -> Option<Vec<u64>> {
        matches!(frame, Frame::Binary(_)).then(|| self.capturer_ids())
    }
}
