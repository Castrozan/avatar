use std::fs;
use std::path::Path;

use avatar_adapters::commands::{RoomFloor, TalkProcess};
use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::ports::{AgentPane, Floor};
use avatar_listening::conversation_addressee::Addressee;

use crate::tools::{TestResult, Tools};

const LISTING: &str = r#"echo '{"a": {"paneId": "p1", "servant": "Ada"}}'"#;
const RECORDED: &str = r#"{"pane":"p1","servant":"Ada","heard_through":7,"host":null}"#;

fn floor(tools: &Tools, directory: &Path) -> Result<RoomFloor, std::io::Error> {
    let characters = tools.path().join("characters");
    fs::write(&characters, "Ada | Violet\nBo | Green\nAda | Amber \n")?;
    let process = TalkProcess::new(tools.programs());
    Ok(RoomFloor::new(directory, process, Some(characters)))
}

fn recorded(directory: &Path) -> Result<String, std::io::Error> {
    fs::read_to_string(directory.join("addressee.json"))
}

fn hand_over_twice(floor: &mut RoomFloor) -> [Result<(), TalkFailure>; 2] {
    [
        floor.give_the_room_to(&addressee("Ada")),
        floor.give_the_room_to(&addressee("Ada")),
    ]
}

fn addressee(servant: &str) -> Addressee {
    let (pane, host) = ("p1".to_owned(), None);
    Addressee {
        pane,
        servant: servant.to_owned(),
        heard_through: 7,
        host,
    }
}

fn agent(name: &str) -> AgentPane {
    AgentPane {
        host: None,
        pane: Some(name.to_owned()),
    }
}

#[test]
fn the_room_is_recorded_and_the_last_listed_character_is_shown_once_per_servant() -> TestResult {
    let tools = Tools::with(&[("avatar-command", "echo \"$@\" >> \"$0.log\"")])?;
    let state = tempfile::tempdir()?;
    let mut floor = floor(&tools, state.path())?;
    assert_eq!(hand_over_twice(&mut floor), [Ok(()), Ok(())]);
    assert_eq!(recorded(state.path())?, RECORDED);
    assert_eq!(tools.read("avatar-command.log"), "character Amber\n");
    Ok(())
}

#[test]
fn a_remote_agent_without_a_servant_does_not_receive_the_room() -> TestResult {
    let tools = Tools::with(&[("a2a", LISTING)])?;
    let state = tempfile::tempdir()?;
    let mut floor = floor(&tools, state.path())?;
    assert_eq!(floor.hand_to_remote_agent(&agent("p9")), Ok(false));
    assert!(!state.path().join("addressee.json").exists());
    Ok(())
}

#[test]
fn a_remote_agent_with_a_servant_receives_the_room_from_the_start_of_the_log() -> TestResult {
    let tools = Tools::with(&[("a2a", LISTING), ("avatar-command", "exit 1")])?;
    let state = tempfile::tempdir()?;
    let mut floor = floor(&tools, state.path())?;
    assert_eq!(floor.hand_to_remote_agent(&agent("p1")), Ok(true));
    let recorded = fs::read_to_string(state.path().join("addressee.json"))?;
    assert_eq!(
        recorded,
        RECORDED.replace("\"heard_through\":7", "\"heard_through\":0")
    );
    Ok(())
}
