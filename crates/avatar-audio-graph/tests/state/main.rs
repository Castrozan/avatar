use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use avatar_audio_graph::pulse::module_record::ModuleRecord;
use avatar_audio_graph::state::file::StateFile;
use avatar_audio_graph::state::record::{AudioState, MicrophoneRecord};
use avatar_audio_graph::state::store::StateStore;
use tempfile::TempDir;

fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn sample_microphone() -> MicrophoneRecord {
    MicrophoneRecord {
        source: "synthetic-human.monitor".to_owned(),
        module: ModuleRecord::new(
            9,
            "module-loopback",
            "source=synthetic-human.monitor".to_owned(),
        ),
    }
}

fn sample_state() -> AudioState {
    let mix = ModuleRecord::new(
        7,
        "module-null-sink",
        "sink_name=AvatarMix rate=48000".to_owned(),
    );
    AudioState {
        modules: vec![mix],
        ready: true,
        microphone: Some(sample_microphone()),
    }
}

fn mode_bits(path: &Path) -> Result<u32, String> {
    Ok(fs::metadata(path).map_err(text)?.permissions().mode() & 0o777)
}

#[test]
fn a_missing_state_file_loads_as_the_empty_graph() -> Result<(), String> {
    let directory = TempDir::new().map_err(text)?;
    let loaded = StateFile::in_directory(directory.path())
        .load()
        .map_err(text)?;
    assert_eq!(loaded, AudioState::default());
    Ok(())
}

#[test]
fn saved_state_round_trips_and_is_owner_only() -> Result<(), String> {
    let directory = TempDir::new().map_err(text)?;
    let mut file = StateFile::in_directory(directory.path());
    file.save(&sample_state()).map_err(text)?;
    assert_eq!(file.load().map_err(text)?, sample_state());
    assert_eq!(mode_bits(&file.path())?, 0o600);
    Ok(())
}

#[test]
fn the_saved_json_keeps_the_python_key_names() -> Result<(), String> {
    let directory = TempDir::new().map_err(text)?;
    let mut file = StateFile::in_directory(directory.path());
    file.save(&sample_state()).map_err(text)?;
    let saved = fs::read_to_string(file.path()).map_err(text)?;
    let value: serde_json::Value = serde_json::from_str(&saved).map_err(text)?;
    assert_eq!(value["ready"], true);
    assert_eq!(value["microphone"]["source"], "synthetic-human.monitor");
    assert_eq!(value["microphone"]["index"], 9);
    assert_eq!(value["modules"][0]["name"], "module-null-sink");
    Ok(())
}

#[test]
fn remove_is_idempotent_and_leaves_no_state_behind() -> Result<(), String> {
    let directory = TempDir::new().map_err(text)?;
    let mut file = StateFile::in_directory(directory.path());
    file.save(&sample_state()).map_err(text)?;
    file.remove().map_err(text)?;
    file.remove().map_err(text)?;
    assert!(!file.path().exists());
    Ok(())
}

#[test]
fn the_exclusive_lock_creates_its_owner_only_directory() -> Result<(), String> {
    let parent = TempDir::new().map_err(text)?;
    let runtime = parent.path().join("avatar");
    let _lock = StateFile::in_directory(&runtime)
        .lock_exclusively()
        .map_err(text)?;
    assert_eq!(mode_bits(&runtime)?, 0o700);
    Ok(())
}
