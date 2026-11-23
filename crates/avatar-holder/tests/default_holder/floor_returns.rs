use super::{cases, holder_of_record, optional_string, record_of};
use avatar_holder::default_holder::{DefaultHolder, floor_returns_to};
use serde_json::Value;
use std::error::Error;

const FIXTURE: &str = include_str!("fixtures/default_holder_floor_returns.json");

fn returned_for(case: &Value) -> Option<DefaultHolder> {
    let configured = holder_of_record(&case["configured"]);
    let current_pane = optional_string(&case["current_pane"]);
    let pane = case["pane"].as_str().unwrap_or_default();
    floor_returns_to(configured.as_ref(), current_pane.as_deref(), pane).cloned()
}

#[test]
fn floor_returns_to_matches_the_python_oracle() -> Result<(), Box<dyn Error>> {
    for case in cases(FIXTURE)? {
        let returned = returned_for(&case);
        assert_eq!(
            record_of(returned.as_ref()),
            case["expected"]["ok"],
            "case {}",
            case["name"]
        );
    }
    Ok(())
}
