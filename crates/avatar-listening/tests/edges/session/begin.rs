use std::path::Path;

use super::fake_units::FakeUnits;
use super::replay_all;
use crate::edges::helpers::{Failure, event_rows, mode_text, strings};
use crate::support::{Outcome, text};
use avatar_listening::conversation_addressee::Addressee;
use avatar_listening::conversation_addressee::file_store::FileAddresseeStore;
use avatar_listening::conversation_addressee::store::AddresseeStore;
use avatar_listening::conversation_state::ListenerPurpose;
use avatar_listening::listener::session::{SessionDirectory, SessionRequest, begin_listening};
use serde_json::{Map, Value};

fn purpose_of(name: &str) -> ListenerPurpose {
    match name {
        "conversation" => ListenerPurpose::Conversation,
        "meeting" => ListenerPurpose::Meeting,
        _ => ListenerPurpose::Live,
    }
}

fn holder() -> Addressee {
    Addressee {
        pane: "w1:p1".into(),
        servant: "Sasaki".into(),
        heard_through: 3,
        host: None,
    }
}

fn existing_files(directory: &Path, case: &Value) -> Outcome {
    for name in strings(&case["existing"])? {
        std::fs::write(directory.join(name), "old")?;
    }
    let store = FileAddresseeStore::new(directory);
    let hand = |()| store.hand_to(&holder());
    (case["addressee"] == true)
        .then_some(())
        .map(hand)
        .transpose()?;
    Ok(())
}

fn existing_state(directory: &Path, case: &Value) -> Outcome {
    std::fs::create_dir(directory)?;
    let mode = std::os::unix::fs::PermissionsExt::from_mode(0o755);
    std::fs::set_permissions(directory, mode)?;
    existing_files(directory, case)
}

fn request_of(case: &Value) -> Result<SessionRequest, Failure> {
    let sources = serde_json::json!({"meeting": "m", "human": "h"});
    Ok(SessionRequest {
        purpose: purpose_of(text(&case["purpose"])?),
        sources: sources.as_object().cloned().unwrap_or_default(),
        settings: case["settings"]
            .as_object()
            .cloned()
            .unwrap_or_else(Map::new),
    })
}

fn file_modes(directory: &Path) -> Result<Value, Failure> {
    let mut modes = Map::new();
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        modes.insert(name.unwrap_or_default(), mode_text(&path)?.into());
    }
    Ok(Value::Object(modes))
}

fn check_results(case: &Value, path: &Path, units: &FakeUnits) -> Outcome {
    assert_eq!(units.events, event_rows(&case["events"])?, "{case}");
    assert_eq!(file_modes(path)?, case["files"], "{case}");
    let written = serde_json::from_slice::<Value>(&std::fs::read(path.join("listener.json"))?)?;
    assert_eq!(written, case["configuration"], "{case}");
    assert_eq!(mode_text(path)?, text(&case["directory_mode"])?, "{case}");
    Ok(())
}

fn replay(case: &Value) -> Outcome {
    let root = tempfile::tempdir()?;
    let path = root.path().join("state");
    if case["fresh"] != true {
        existing_state(&path, case)?;
    }
    let mut units = FakeUnits::with_active(strings(&case["active"])?);
    let directory = SessionDirectory::new(&path);
    begin_listening(&directory, &mut units, &request_of(case)?)?;
    check_results(case, &path, &units)
}

#[test]
fn every_oracle_begin_matches_the_python_session() -> Outcome {
    replay_all("begin", 14, replay)
}
