use avatar_floor::addressee::{Addressee, current, hand_to};
use avatar_floor::floor::{FloorBoundaries, RemoteAgent, ServantCharacter, Session, claim};
use std::cell::RefCell;
use std::error::Error;
use std::io;
use std::path::Path;
use tempfile::{TempDir, tempdir};

pub(crate) struct FakeFloor {
    pub(crate) sessions: Vec<Session>,
    pub(crate) listening: bool,
}

impl FloorBoundaries for FakeFloor {
    fn sessions(&self) -> Vec<Session> {
        self.sessions.clone()
    }

    fn listening(&self) -> bool {
        self.listening
    }
}

#[derive(Default)]
pub(crate) struct RecordingCharacter {
    shown: RefCell<Vec<String>>,
}

impl RecordingCharacter {
    pub(crate) fn shown(&self) -> Vec<String> {
        self.shown.borrow().clone()
    }
}

impl ServantCharacter for RecordingCharacter {
    fn show_for(&self, servant: &str) {
        self.shown.borrow_mut().push(servant.to_owned());
    }
}

pub(crate) fn session(pane: &str, servant: &str, host: Option<&str>) -> Session {
    Session {
        servant: servant.to_owned(),
        pane: pane.to_owned(),
        host: host.map(str::to_owned),
    }
}

pub(crate) fn addressee(
    pane: &str,
    servant: &str,
    heard_through: i64,
    host: Option<&str>,
) -> Addressee {
    Addressee {
        pane: pane.to_owned(),
        servant: servant.to_owned(),
        heard_through,
        host: host.map(str::to_owned),
    }
}

pub(crate) fn directory_holding(holder: Option<&Addressee>) -> Result<TempDir, Box<dyn Error>> {
    let directory = tempdir()?;
    if let Some(held) = holder {
        hand_to(directory.path(), held)?;
    }
    Ok(directory)
}

pub(crate) fn claim_on(
    directory: &Path,
    pane: &str,
    sessions: Vec<Session>,
    listening: bool,
) -> io::Result<Option<Addressee>> {
    claim(
        directory,
        pane,
        &FakeFloor {
            sessions,
            listening,
        },
    )
}

pub(crate) fn assert_stored(
    directory: &Path,
    expected: Option<&Addressee>,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(current(directory)?.as_ref(), expected);
    Ok(())
}

pub(crate) fn remote_agent() -> RemoteAgent {
    RemoteAgent {
        pane: "p7".to_owned(),
        host: Some("box".to_owned()),
    }
}

pub(crate) fn answering(servant: Option<&'static str>) -> impl Fn(&RemoteAgent) -> Option<String> {
    move |_agent| servant.map(str::to_owned)
}

pub(crate) fn stored_json(directory: &Path) -> Result<serde_json::Value, Box<dyn Error>> {
    let text = std::fs::read_to_string(directory.join("addressee.json"))?;
    Ok(serde_json::from_str(&text)?)
}
