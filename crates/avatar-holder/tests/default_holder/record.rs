use super::{cases, parsed_matches};
use avatar_holder::default_holder::{CONFIGURATION_KEY, configured_holder, holder_from_record};
use serde_json::{Map, Value, json};
use std::error::Error;

const FIXTURE: &str = include_str!("fixtures/default_holder_record.json");

fn configuration_with(record: &Value) -> Map<String, Value> {
    let mut configuration = Map::new();
    configuration.insert(CONFIGURATION_KEY.to_owned(), record.clone());
    configuration
}

#[test]
fn configured_holder_matches_the_python_oracle() -> Result<(), Box<dyn Error>> {
    for case in cases(FIXTURE)? {
        let parsed = configured_holder(&configuration_with(&case["record"]));
        assert!(
            parsed_matches(&parsed, &case["expected"]),
            "case {}",
            case["name"]
        );
    }
    Ok(())
}

#[test]
fn a_missing_configuration_key_is_no_holder() {
    assert_eq!(configured_holder(&Map::new()), Ok(None));
}

#[test]
fn a_record_with_unknown_keys_is_rejected() {
    assert!(holder_from_record(&json!({"profile": "silver", "extra": 1})).is_err());
}
