use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use avatar_listening::conversation_addressee::Addressee;
use avatar_listening::conversation_addressee::file_store::{ADDRESSEE_FILE, FileAddresseeStore};
use avatar_listening::conversation_addressee::store::AddresseeStore;
use serde_json::Value;
use tempfile::TempDir;

use crate::support::{Outcome, flag, optional_text, replay_cases, rows, text};

const STORE_FIXTURE: &str = "conversation_addressee/fixtures/store.json";
const STAGED_FILE: &str = ".addressee.json";

fn addressee_from_row(fields: &[Value]) -> Result<Addressee, Box<dyn std::error::Error>> {
    Ok(Addressee {
        pane: text(&fields[1])?.to_owned(),
        servant: text(&fields[2])?.to_owned(),
        heard_through: fields[3].as_i64().ok_or("heard_through")?,
        host: optional_text(&fields[4])?.map(str::to_owned),
    })
}

fn addressee_from_object(object: &Value) -> Result<Option<Addressee>, Box<dyn std::error::Error>> {
    if object.is_null() {
        return Ok(None);
    }
    Ok(Some(Addressee {
        pane: text(&object["pane"])?.to_owned(),
        servant: text(&object["servant"])?.to_owned(),
        heard_through: object["heard_through"].as_i64().ok_or("heard_through")?,
        host: optional_text(&object["host"])?.map(str::to_owned),
    }))
}

fn apply(store: &FileAddresseeStore, step: &Value) -> Outcome {
    let fields = rows(step)?;
    if text(&fields[0])? == "forget" {
        store.forget()?;
    } else {
        store.hand_to(&addressee_from_row(fields)?)?;
    }
    Ok(())
}

fn file_on_disk(directory: &Path) -> Result<Option<Value>, Box<dyn std::error::Error>> {
    match fs::read_to_string(directory.join(ADDRESSEE_FILE)) {
        Ok(contents) => Ok(Some(serde_json::from_str(&contents)?)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn check_file_and_staging(directory: &Path, observed: &Value) -> Outcome {
    let expected_file = (!observed["file"].is_null()).then(|| observed["file"].clone());
    assert_eq!(file_on_disk(directory)?, expected_file);
    let staged = directory.join(STAGED_FILE).exists();
    assert_eq!(staged, flag(&observed["staged_left_behind"])?);
    Ok(())
}

fn check_observation(directory: &Path, store: &FileAddresseeStore, observed: &Value) -> Outcome {
    assert_eq!(
        store.current()?,
        addressee_from_object(&observed["current"])?
    );
    check_file_and_staging(directory, observed)
}

fn replay_step(
    directory: &Path,
    store: &FileAddresseeStore,
    step: Option<&Value>,
    observed: &Value,
) -> Outcome {
    if let Some(step) = step {
        apply(store, step)?;
    }
    check_observation(directory, store, observed)
}

fn replay(case: &Value) -> Outcome {
    let root = TempDir::new()?;
    let directory = root.path().join("state");
    let store = FileAddresseeStore::new(&directory);
    let steps = rows(&case["steps"])?;
    for (index, observed) in rows(&case["observed"])?.iter().enumerate() {
        replay_step(&directory, &store, steps.get(index), observed)?;
    }
    Ok(())
}

#[test]
fn every_oracle_store_history_matches_the_python_addressee_files() -> Outcome {
    replay_cases(STORE_FIXTURE, "cases", 16, replay)
}
