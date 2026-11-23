use std::io;
use std::path::{Path, PathBuf};

use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::ports::exec::run_unchecked;
use avatar_conversation::talk::ports::{AgentPane, Floor};
use avatar_listening::conversation_addressee::Addressee;
use avatar_listening::conversation_addressee::file_store::FileAddresseeStore;
use avatar_listening::conversation_addressee::store::AddresseeStore;

use super::a2a::A2aLookup;
use super::process::TalkProcess;

const SWITCH_SECONDS: f64 = 30.0;
const SEPARATOR: &str = " | ";

pub struct RoomFloor {
    store: FileAddresseeStore,
    lookup: A2aLookup,
    process: TalkProcess,
    characters_file: Option<PathBuf>,
}

impl RoomFloor {
    #[must_use]
    pub fn new(
        directory: impl Into<PathBuf>,
        process: TalkProcess,
        characters_file: Option<PathBuf>,
    ) -> Self {
        Self {
            store: FileAddresseeStore::new(directory),
            lookup: A2aLookup::new(process.clone()),
            process,
            characters_file,
        }
    }
}

fn failure_of(error: &io::Error) -> TalkFailure {
    TalkFailure::new(error.to_string())
}

fn character_of(characters_file: Option<&Path>, servant: &str) -> Option<String> {
    let text = std::fs::read_to_string(characters_file?).ok()?;
    let mut listed = text.lines().filter_map(|line| line.split_once(SEPARATOR));
    let found = listed.rfind(|(name, _)| *name == servant);
    found.map(|(_, character)| character.trim().to_owned())
}

fn show_character_for(process: &mut TalkProcess, characters_file: Option<&Path>, servant: &str) {
    let Some(character) = character_of(characters_file, servant) else {
        return;
    };
    let argv = ["avatar-command", "character", &character];
    let _ = run_unchecked(process, &argv, Some(SWITCH_SECONDS));
}

fn hand_over(floor: &mut RoomFloor, addressee: &Addressee) -> Result<(), TalkFailure> {
    let previous = floor.store.current().map_err(|error| failure_of(&error))?;
    let handed = floor.store.hand_to(addressee);
    handed.map_err(|error| failure_of(&error))?;
    if previous.is_none_or(|held| held.servant != addressee.servant) {
        let file = floor.characters_file.as_deref();
        show_character_for(&mut floor.process, file, &addressee.servant);
    }
    Ok(())
}

fn addressee_for(agent: &AgentPane, servant: String) -> Addressee {
    Addressee {
        pane: agent.pane.clone().unwrap_or_default(),
        servant,
        heard_through: 0,
        host: agent.host.clone(),
    }
}

fn hand_to_servant(floor: &mut RoomFloor, agent: &AgentPane) -> Result<bool, TalkFailure> {
    let servant = floor.lookup.servant_at(agent)?;
    let Some(name) = servant.filter(|name| !name.is_empty()) else {
        return Ok(false);
    };
    hand_over(floor, &addressee_for(agent, name)).map(|()| true)
}

impl Floor for RoomFloor {
    fn give_the_room_to(&mut self, addressee: &Addressee) -> Result<(), TalkFailure> {
        hand_over(self, addressee)
    }

    fn hand_to_remote_agent(&mut self, agent: &AgentPane) -> Result<bool, TalkFailure> {
        hand_to_servant(self, agent)
    }
}
