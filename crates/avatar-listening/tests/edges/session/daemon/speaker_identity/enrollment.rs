use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::vector_of;
use crate::support::{Outcome, count, fixture, rows, text};
use avatar_listening::listener::speaker_identity::{VoiceProfiles, append_embedding, enroll};
use serde_json::Value;

fn mode_of(path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
    Ok(u64::from(
        std::fs::metadata(path)?.permissions().mode() & 0o777,
    ))
}

fn seed_embeddings(root: &Path, case: &Value) -> Outcome {
    for entry in rows(&case["entries"])? {
        let sequence = i64::try_from(count(&entry[0])?)?;
        append_embedding(
            &root.join("embeddings.jsonl"),
            sequence,
            &vector_of(&entry[1])?,
        )?;
    }
    Ok(())
}

fn seed_profile(profiles: &VoiceProfiles, case: &Value) -> Outcome {
    let existing = &case["existing"];
    let Some(name) = existing["name"].as_str() else {
        return Ok(());
    };
    let samples = existing["samples"].as_i64().ok_or("samples")?;
    Ok(profiles.save(name, &vector_of(&existing["embedding"])?, samples)?)
}

fn assert_file(path: &Path, expected: &Value) -> Outcome {
    assert_eq!(std::fs::read_to_string(path)?, text(&expected["text"])?);
    assert_eq!(Some(mode_of(path)?), expected["mode"].as_u64());
    Ok(())
}

fn assert_voices(directory: &Path, case: &Value) -> Outcome {
    let stored = case["voices"].as_object().ok_or("voices")?;
    for (name, expected) in stored {
        assert_file(&directory.join(name), expected)?;
    }
    let present = std::fs::read_dir(directory).map_or(0, Iterator::count);
    assert_eq!(present, stored.len());
    Ok(())
}

fn assert_embeddings_file(path: &Path, case: &Value) {
    let written = std::fs::read_to_string(path).ok();
    assert_eq!(written.as_deref(), case["embeddings_file"].as_str());
    assert_eq!(mode_of(path).ok(), case["embeddings_mode"].as_u64());
}

fn assert_outcome(outcome: Result<i64, impl ToString>, case: &Value) {
    let result = &case["result"];
    let message = outcome.as_ref().err().map(ToString::to_string);
    assert_eq!(message.as_deref(), result["error"].as_str());
    assert_eq!(outcome.ok(), result["samples"].as_i64());
}

fn range_of(case: &Value) -> Result<(i64, i64), Box<dyn std::error::Error>> {
    let first = case["first"].as_i64().ok_or("first")?;
    Ok((first, case["last"].as_i64().ok_or("last")?))
}

fn replay(case: &Value) -> Outcome {
    let root = tempfile::tempdir()?;
    let profiles = VoiceProfiles::new(root.path().join("voices"));
    seed_embeddings(root.path(), case)?;
    seed_profile(&profiles, case)?;
    let path = root.path().join("embeddings.jsonl");
    let outcome = enroll(&profiles, &path, text(&case["name"])?, range_of(case)?);
    assert_outcome(outcome, case);
    assert_voices(&root.path().join("voices"), case)?;
    assert_embeddings_file(&path, case);
    Ok(())
}

#[test]
fn enrolling_matches_the_python_profiles_and_files() -> Outcome {
    let document = fixture("edges/session/daemon/speaker_identity/fixtures/oracle.json")?;
    let cases = rows(&document["enroll"])?;
    assert_eq!(cases.len(), 25);
    cases.iter().try_for_each(replay)
}
