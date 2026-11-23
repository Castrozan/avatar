use crate::support::{Outcome, fixture, flag, optional_text, rows, text};
use avatar_listening::conversation_state::{ListenerPurpose, command_of, decide, observe};
mod names;
mod properties;

use names::{ACTIONS, COMMANDS, PHASES, name_of, named};
use serde_json::Value;

fn purpose_named(
    name: Option<&str>,
) -> Result<Option<ListenerPurpose>, Box<dyn std::error::Error>> {
    match name {
        None => Ok(None),
        Some("conversation") => Ok(Some(ListenerPurpose::Conversation)),
        Some("meeting") => Ok(Some(ListenerPurpose::Meeting)),
        Some("live") => Ok(Some(ListenerPurpose::Live)),
        Some(other) => Err(format!("unknown purpose {other}").into()),
    }
}

fn transition_row(row: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let phase = named(&PHASES, text(&row[0])?)?;
    let command = named(&COMMANDS, text(&row[1])?)?;
    let action = decide(
        phase,
        command,
        optional_text(&row[2])?,
        optional_text(&row[3])?,
    );
    assert_eq!(name_of(&ACTIONS, action), text(&row[4])?, "{row}");
    Ok(())
}

#[test]
fn every_oracle_transition_matches_the_python_table() -> Outcome {
    let document = fixture("conversation_state/fixtures/transitions.json")?;
    let transitions = rows(&document["transitions"])?;
    assert_eq!(transitions.len(), 495);
    for row in transitions {
        transition_row(row)?;
    }
    Ok(())
}

#[test]
fn every_oracle_observation_matches_the_python_phase() -> Outcome {
    let document = fixture("conversation_state/fixtures/transitions.json")?;
    for row in rows(&document["observe"])? {
        let purpose = purpose_named(optional_text(&row[1])?)?;
        let phase = observe(flag(&row[0])?, purpose);
        assert_eq!(name_of(&PHASES, phase), text(&row[2])?, "{row}");
    }
    Ok(())
}

#[test]
fn every_oracle_command_resolves_like_the_python_arguments() -> Outcome {
    let document = fixture("conversation_state/fixtures/transitions.json")?;
    for row in rows(&document["command_of"])? {
        let action = named(&COMMANDS, text(&row[1])?)?;
        let command = command_of(flag(&row[0])?, action);
        assert_eq!(name_of(&COMMANDS, command), text(&row[2])?, "{row}");
    }
    Ok(())
}
