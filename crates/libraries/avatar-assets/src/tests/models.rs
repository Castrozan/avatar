use std::fs;
use std::io::ErrorKind;

use super::defaults::{TestResult, environment_over};
use crate::{Asset, resolve_in};

const MODELS: [(Asset, &str); 3] = [
    (Asset::TurnModel, "models/smart-turn-v3.2-cpu.onnx"),
    (Asset::VoiceActivityModel, "models/silero_vad.onnx"),
    (Asset::SpeakerModel, "models/speaker-embedding.onnx"),
];

#[test]
fn a_missing_model_points_at_the_setup_command() -> TestResult {
    let directory = tempfile::tempdir()?;
    let environment = environment_over(&directory, &[]);
    for (asset, _) in MODELS {
        let error = resolve_in(asset, &environment).err().ok_or("found")?;
        assert_eq!(error.kind(), ErrorKind::NotFound);
        assert!(error.to_string().contains("avatar setup"));
        assert!(error.to_string().contains(asset.variable()));
    }
    Ok(())
}

#[test]
fn a_model_in_the_data_directory_is_found_without_variables() -> TestResult {
    let directory = tempfile::tempdir()?;
    fs::create_dir_all(directory.path().join("models"))?;
    let environment = environment_over(&directory, &[]);
    for (asset, relative) in MODELS {
        let expected = directory.path().join(relative);
        fs::write(&expected, b"model")?;
        assert_eq!(resolve_in(asset, &environment)?, expected);
    }
    Ok(())
}

#[test]
fn a_model_variable_wins_over_the_data_directory() -> TestResult {
    let directory = tempfile::tempdir()?;
    let environment = environment_over(&directory, &[("AVATAR_TURN_MODEL", "/store/turn.onnx")]);
    let path = resolve_in(Asset::TurnModel, &environment)?;
    assert_eq!(path.to_str(), Some("/store/turn.onnx"));
    Ok(())
}

const ASSETS: [Asset; 9] = [
    Asset::RendererRoot,
    Asset::SeatPage,
    Asset::WakePage,
    Asset::CharacterVoices,
    Asset::ServantCharacters,
    Asset::TurnModel,
    Asset::VoiceActivityModel,
    Asset::SpeakerModel,
    Asset::VoiceDirectory,
];

#[test]
fn every_asset_variable_is_distinct_and_prefixed() {
    let mut names: Vec<_> = ASSETS.iter().map(|asset| asset.variable()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), ASSETS.len());
    assert!(names.iter().all(|name| name.starts_with("AVATAR_")));
}
