use avatar_floor::addressee::{
    Addressee, addresses, current, forget, hand_to, hermes_pane, holds_without_a_pane,
};
use tempfile::tempdir;

fn ruri() -> Addressee {
    Addressee {
        pane: "pane-1".to_owned(),
        servant: "Ruri".to_owned(),
        heard_through: 7,
        host: None,
    }
}

#[test]
fn nothing_is_current_in_an_empty_directory() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    assert_eq!(current(directory.path())?, None);
    Ok(())
}

#[test]
fn a_handed_addressee_is_current_and_stored_as_python_stores_it()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    hand_to(directory.path(), &ruri())?;
    let text = std::fs::read_to_string(directory.path().join("addressee.json"))?;
    let expected = r#"{"pane":"pane-1","servant":"Ruri","heard_through":7,"host":null}"#;
    assert_eq!(text, expected);
    assert_eq!(current(directory.path())?, Some(ruri()));
    Ok(())
}

#[test]
fn forgetting_removes_the_addressee_and_is_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    hand_to(directory.path(), &ruri())?;
    forget(directory.path())?;
    forget(directory.path())?;
    assert_eq!(current(directory.path())?, None);
    Ok(())
}

#[test]
fn addresses_follows_the_pane_rule() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    assert!(addresses(directory.path(), Some("any"))?);
    hand_to(directory.path(), &ruri())?;
    assert!(addresses(directory.path(), Some("pane-1"))?);
    assert!(addresses(directory.path(), None)?);
    assert!(!addresses(directory.path(), Some("pane-2"))?);
    Ok(())
}

#[test]
fn hermes_panes_hold_without_a_pane() {
    let hermes = Addressee {
        pane: hermes_pane("silver"),
        ..ruri()
    };
    assert!(holds_without_a_pane(&hermes));
    assert!(!holds_without_a_pane(&ruri()));
}
