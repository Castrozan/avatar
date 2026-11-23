use super::cases;
use avatar_holder::default_holder::listener_settings;
use serde_json::{Value, json};
use std::error::Error;

const FIXTURE: &str = include_str!("fixtures/default_holder_settings.json");

fn arguments_of(name: &str) -> (Option<&'static str>, Option<&'static str>) {
    match name {
        "no holder" => (None, Some("x")),
        "holder with character" => (Some("silver"), Some("darkness-shibu")),
        "holder without character" => (Some("silver"), None),
        _ => (Some(""), None),
    }
}

#[test]
fn listener_settings_match_the_python_oracle() -> Result<(), Box<dyn Error>> {
    for case in cases(FIXTURE)? {
        let name = case["name"].as_str().unwrap_or_default();
        let (profile, character) = arguments_of(name);
        let actual = Value::Object(listener_settings(profile, character));
        assert_eq!(actual, case["expected"], "case {name}");
    }
    assert_eq!(json!({}), Value::Object(listener_settings(None, None)));
    Ok(())
}
