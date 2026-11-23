use crate::support::{Outcome, fixture, flag, optional_text, replay_cases, rows, text};
use avatar_listening::conversation_addressee::{
    Addressee, addresses, hermes_pane, holder_other_than, holds_without_a_pane,
};
use serde_json::Value;

const ADDRESSEE_FIXTURE: &str = "conversation_addressee/fixtures/addressee.json";

fn addressee_of(holder: &Value) -> Result<Option<Addressee>, Box<dyn std::error::Error>> {
    if holder.is_null() {
        return Ok(None);
    }
    let fields = rows(holder)?;
    Ok(Some(Addressee {
        pane: text(&fields[0])?.to_owned(),
        servant: text(&fields[1])?.to_owned(),
        heard_through: fields[2].as_i64().ok_or("heard_through")?,
        host: optional_text(&fields[3])?.map(str::to_owned),
    }))
}

fn check_found(found: &Addressee, expected: &Value, row: &Value) -> Outcome {
    assert_eq!(found.pane, text(&expected["pane"])?, "{row}");
    assert_eq!(
        found.heard_through,
        expected["heard_through"].as_i64().unwrap_or_default(),
        "{row}"
    );
    Ok(())
}

fn check_other(other: Option<&Addressee>, expected: &Value, row: &Value) -> Outcome {
    assert_eq!(other.is_none(), expected.is_null(), "{row}");
    if let Some(found) = other {
        check_found(found, expected, row)?;
    }
    Ok(())
}

fn holder_row(row: &Value) -> Outcome {
    let holder = addressee_of(&row["holder"])?;
    let pane = optional_text(&row["pane"])?;
    let other = holder_other_than(holder.as_ref(), pane);
    check_other(other, &row["holder_other_than"], row)?;
    assert_eq!(
        addresses(holder.as_ref(), pane),
        flag(&row["addresses"])?,
        "{row}"
    );
    Ok(())
}

fn named_holder(pane: &str) -> Addressee {
    Addressee {
        pane: pane.to_owned(),
        servant: "s".to_owned(),
        heard_through: 0,
        host: None,
    }
}

fn hold_row(row: &Value) -> Outcome {
    let holder = named_holder(text(&row[0])?);
    assert_eq!(holds_without_a_pane(&holder), flag(&row[1])?, "{row}");
    Ok(())
}

#[test]
fn every_oracle_holder_matches_the_python_addressee() -> Outcome {
    replay_cases(ADDRESSEE_FIXTURE, "addressee", 12, holder_row)
}

#[test]
fn every_oracle_hermes_pane_matches_the_python_profile_rule() -> Outcome {
    let document = fixture(ADDRESSEE_FIXTURE)?;
    for row in rows(&document["hermes"])? {
        assert_eq!(hermes_pane(text(&row[0])?), text(&row[1])?);
    }
    Ok(())
}

#[test]
fn only_hermes_panes_hold_without_a_pane() -> Outcome {
    replay_cases(ADDRESSEE_FIXTURE, "holds_without_a_pane", 2, hold_row)
}
