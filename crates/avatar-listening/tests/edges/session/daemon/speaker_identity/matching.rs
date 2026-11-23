use std::path::Path;

use super::vector_of;
use crate::support::{Outcome, fixture, rows, text};
use avatar_listening::listener::speaker_identity::VoiceProfiles;
use serde_json::{Value, json};

fn stored(root: &Path, case: &Value) -> Result<VoiceProfiles, Box<dyn std::error::Error>> {
    let profiles = VoiceProfiles::new(root.join("v"));
    for entry in rows(&case["profiles"])? {
        let samples = entry[2].as_i64().ok_or("samples")?;
        profiles.save(text(&entry[0])?, &vector_of(&entry[1])?, samples)?;
    }
    Ok(profiles)
}

fn forgetting_message(profiles: &VoiceProfiles, name: &str) -> String {
    let result = profiles.forget(name);
    result.map_or_else(|failure| failure.to_string(), |()| "forgotten".to_owned())
}

fn assert_forgetting(profiles: &VoiceProfiles, case: &Value) -> Outcome {
    let forget = &case["forget"];
    for (name, expected) in rows(&forget[0])?.iter().zip(rows(&forget[1])?) {
        assert_eq!(forgetting_message(profiles, text(name)?), text(expected)?);
    }
    assert_eq!(json!(profiles.names()?), forget[2]);
    Ok(())
}

fn assert_strays(profiles: &VoiceProfiles, root: &Path, case: &Value, query: &[f64]) -> Outcome {
    std::fs::create_dir_all(root.join("v"))?;
    std::fs::write(root.join("v/stray.txt"), "x")?;
    std::fs::write(root.join("v/weird name.json"), "{}")?;
    assert_eq!(json!(profiles.names()?), case["names_with_strays"]);
    let outcome = profiles.best_match(query);
    let expected = &case["best_with_strays"];
    let message = outcome.as_ref().err().map(ToString::to_string);
    assert_eq!(message.as_deref(), expected["error"].as_str());
    assert_eq!(outcome.ok().flatten().as_deref(), expected["name"].as_str());
    Ok(())
}

fn assert_refusal(profiles: &VoiceProfiles, case: &Value) {
    let refusal = profiles
        .load("../x")
        .err()
        .map(|failure| failure.to_string());
    assert_eq!(refusal.as_deref(), case["load_../x"].as_str());
}

fn assert_loading(profiles: &VoiceProfiles, case: &Value) -> Outcome {
    for name in rows(&case["names"])? {
        let loaded = profiles.load(text(name)?)?.ok_or("profile")?;
        let expected = case["loaded"][text(name)?]["samples"].as_i64();
        assert_eq!(Some(loaded.samples), expected);
    }
    assert_eq!(profiles.load("ghost")?, None);
    assert_refusal(profiles, case);
    Ok(())
}

fn replay(case: &Value) -> Outcome {
    let root = tempfile::tempdir()?;
    let profiles = stored(root.path(), case)?;
    let query = vector_of(&case["query"])?;
    let best = profiles.best_match(&query)?;
    assert_eq!(best.as_deref(), case["best"].as_str());
    assert_eq!(json!(profiles.names()?), case["names"]);
    assert_loading(&profiles, case)?;
    assert_forgetting(&profiles, case)?;
    assert_strays(&profiles, root.path(), case, &query)
}

#[test]
fn best_match_names_forgetting_and_strays_match_python() -> Outcome {
    let document = fixture("edges/session/daemon/speaker_identity/fixtures/oracle.json")?;
    let cases = rows(&document["match"])?;
    assert_eq!(cases.len(), 27);
    cases.iter().try_for_each(replay)
}
