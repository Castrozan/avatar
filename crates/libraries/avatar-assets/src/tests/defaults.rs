use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;

use crate::{Asset, ordinary_word_paths, resolve_in};

pub(super) type TestResult = Result<(), Box<dyn std::error::Error>>;

pub(super) fn environment_over(
    directory: &tempfile::TempDir,
    extra: &[(&str, &str)],
) -> HashMap<String, OsString> {
    let mut variables =
        HashMap::from([("AVATAR_DATA_DIRECTORY".to_owned(), directory.path().into())]);
    variables.extend(
        extra
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).into())),
    );
    variables
}

#[test]
fn an_environment_variable_wins_and_writes_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let environment = environment_over(&directory, &[("AVATAR_RENDERER_ROOT", "/chosen")]);
    let root = resolve_in(Asset::RendererRoot, &environment)?;
    assert_eq!(root.to_str(), Some("/chosen"));
    assert_eq!(fs::read_dir(directory.path())?.count(), 0);
    Ok(())
}

#[test]
fn without_variables_the_renderer_comes_from_the_embedded_files() -> TestResult {
    let directory = tempfile::tempdir()?;
    let root = resolve_in(Asset::RendererRoot, &environment_over(&directory, &[]))?;
    assert!(root.join("index.html").is_file());
    assert!(root.join("renderer.js").is_file());
    assert!(root.join("portraits/jarvis.jpeg").is_file());
    let catalog: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("characters.json"))?)?;
    assert_eq!(catalog[0]["name"], "darkness-shibu");
    Ok(())
}

#[test]
fn without_variables_the_pages_come_from_the_embedded_files() -> TestResult {
    let directory = tempfile::tempdir()?;
    let environment = environment_over(&directory, &[]);
    let seat = resolve_in(Asset::SeatPage, &environment)?;
    let wake = resolve_in(Asset::WakePage, &environment)?;
    assert!(seat.join("seat.js").is_file());
    assert!(wake.join("wake-capture.js").is_file());
    Ok(())
}

#[test]
fn without_variables_the_cast_comes_from_the_embedded_tables() -> TestResult {
    let directory = tempfile::tempdir()?;
    let environment = environment_over(&directory, &[]);
    let voices = resolve_in(Asset::CharacterVoices, &environment)?;
    let servants = resolve_in(Asset::ServantCharacters, &environment)?;
    let table: Vec<serde_json::Value> = serde_json::from_str(&fs::read_to_string(voices)?)?;
    assert!(table.iter().all(|entry| entry["liveVoice"].is_string()));
    assert!(fs::read_to_string(servants)?.contains("Achilles | sakurada-fumiriya"));
    Ok(())
}

#[test]
fn the_voice_directory_defaults_under_the_data_directory() -> TestResult {
    let directory = tempfile::tempdir()?;
    let voices = resolve_in(Asset::VoiceDirectory, &environment_over(&directory, &[]))?;
    assert_eq!(voices, directory.path().join("voices"));
    Ok(())
}

#[test]
fn ordinary_words_default_to_the_downloaded_frequency_lists() -> TestResult {
    let directory = tempfile::tempdir()?;
    let paths = ordinary_word_paths(&environment_over(&directory, &[]));
    assert_eq!(paths.split(':').count(), 2);
    assert!(paths.contains(&*directory.path().to_string_lossy()));
    Ok(())
}

#[test]
fn the_ordinary_words_variable_wins() -> TestResult {
    let directory = tempfile::tempdir()?;
    let chosen = environment_over(&directory, &[("AVATAR_ORDINARY_WORDS", "/a:/b")]);
    assert_eq!(ordinary_word_paths(&chosen), "/a:/b");
    Ok(())
}
