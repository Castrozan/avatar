use crate::floor::{FloorBoundaries, ServantCharacter, Session};
use std::cell::RefCell;

pub(super) struct LiveSessions {
    pub(super) sessions: Vec<Session>,
    pub(super) listening: bool,
}

impl FloorBoundaries for LiveSessions {
    fn sessions(&self) -> Vec<Session> {
        self.sessions.clone()
    }

    fn listening(&self) -> bool {
        self.listening
    }
}

#[derive(Default)]
pub(super) struct ShownServants {
    shown: RefCell<Vec<String>>,
}

impl ShownServants {
    pub(super) fn into_shown(self) -> Vec<String> {
        self.shown.into_inner()
    }
}

impl ServantCharacter for ShownServants {
    fn show_for(&self, servant: &str) {
        self.shown.borrow_mut().push(servant.to_owned());
    }
}
