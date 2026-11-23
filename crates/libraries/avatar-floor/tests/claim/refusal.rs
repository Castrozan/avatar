use super::support::{HOST, Listens, Rooms, hand_addressee, read_claim};
use avatar_floor::claim::{Site, claim};
use std::error::Error;
use tempfile::tempdir;

#[test]
fn a_session_talking_on_this_machine_refuses_another_room() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    hand_addressee(directory.path(), "p", None)?;
    let (listening, rooms) = (Listens(true), Rooms::default());
    let site = Site::new(directory.path(), HOST, &listening, &rooms);
    assert!(!claim(&site, "p", "alpha", 1.0)?);
    assert_eq!(read_claim(directory.path(), "p")?, "");
    Ok(())
}

#[test]
fn a_stopped_listener_does_not_refuse() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    hand_addressee(directory.path(), "p", None)?;
    let (listening, rooms) = (Listens(false), Rooms::default());
    let site = Site::new(directory.path(), HOST, &listening, &rooms);
    assert!(claim(&site, "p", "alpha", 1.0)?);
    Ok(())
}

#[test]
fn a_session_never_refuses_its_own_room() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    hand_addressee(directory.path(), "p", None)?;
    let (listening, rooms) = (Listens(true), Rooms::default());
    let site = Site::new(directory.path(), HOST, &listening, &rooms);
    assert!(claim(&site, "p", HOST, 1.0)?);
    Ok(())
}

#[test]
fn a_session_on_a_remote_host_does_not_refuse_here() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    hand_addressee(directory.path(), "p", Some("remote1"))?;
    let (listening, rooms) = (Listens(true), Rooms::default());
    let site = Site::new(directory.path(), HOST, &listening, &rooms);
    assert!(claim(&site, "p", "alpha", 1.0)?);
    Ok(())
}

#[test]
fn an_empty_addressee_host_means_this_host() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    hand_addressee(directory.path(), "p", Some(""))?;
    let (listening, rooms) = (Listens(true), Rooms::default());
    let site = Site::new(directory.path(), HOST, &listening, &rooms);
    assert!(!claim(&site, "p", "alpha", 1.0)?);
    Ok(())
}

#[test]
fn a_session_on_another_pane_does_not_refuse() -> Result<(), Box<dyn Error>> {
    let directory = tempdir()?;
    hand_addressee(directory.path(), "q", None)?;
    let (listening, rooms) = (Listens(true), Rooms::default());
    let site = Site::new(directory.path(), HOST, &listening, &rooms);
    assert!(claim(&site, "p", "alpha", 1.0)?);
    Ok(())
}
