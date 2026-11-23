use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::names::validated_name;
use super::profile_file::{profile_text, write_profile};
use crate::listener::speaker_identity::arithmetic::cosine_similarity;
use crate::listener::speaker_identity::error::VoiceError;

pub const MATCH_THRESHOLD: f64 = 0.55;
const PROFILE_SUFFIX: &str = ".json";

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VoiceProfile {
    pub embedding: Vec<f64>,
    pub samples: i64,
}

pub struct VoiceProfiles {
    directory: PathBuf,
}

fn stem_of(file_name: &str) -> Option<String> {
    let stem = file_name.strip_suffix(PROFILE_SUFFIX)?;
    Some(if stem.is_empty() { file_name } else { stem }.to_owned())
}

fn listed_names(directory: &Path) -> Result<Vec<String>, VoiceError> {
    let entries = match fs::read_dir(directory) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        other => other?,
    };
    let mut names = Vec::new();
    for entry in entries {
        names.extend(stem_of(&entry?.file_name().to_string_lossy()));
    }
    Ok(names)
}

fn read_profile(path: &Path) -> Result<Option<VoiceProfile>, VoiceError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(serde_json::from_str(&text)?)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn closer(best: (Option<String>, f64), name: String, score: f64) -> (Option<String>, f64) {
    if score > best.1 {
        (Some(name), score)
    } else {
        best
    }
}

impl VoiceProfiles {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    fn path(&self, name: &str) -> Result<PathBuf, VoiceError> {
        let valid = validated_name(name)?;
        Ok(self.directory.join(format!("{valid}{PROFILE_SUFFIX}")))
    }

    pub fn names(&self) -> Result<Vec<String>, VoiceError> {
        let mut names = listed_names(&self.directory)?;
        names.sort();
        Ok(names)
    }

    pub fn load(&self, name: &str) -> Result<Option<VoiceProfile>, VoiceError> {
        read_profile(&self.path(name)?)
    }

    pub fn save(&self, name: &str, embedding: &[f64], samples: i64) -> Result<(), VoiceError> {
        let path = self.path(name)?;
        Ok(write_profile(
            &self.directory,
            &path,
            &profile_text(embedding, samples),
        )?)
    }

    pub fn forget(&self, name: &str) -> Result<(), VoiceError> {
        let path = self.path(name)?;
        fs::remove_file(path).map_err(|_| VoiceError::UnknownVoice(name.to_owned()))
    }

    pub fn best_match(&self, embedding: &[f64]) -> Result<Option<String>, VoiceError> {
        closest_voice(self, embedding)
    }
}

fn closest_voice(
    profiles: &VoiceProfiles,
    embedding: &[f64],
) -> Result<Option<String>, VoiceError> {
    let mut best = (None, MATCH_THRESHOLD);
    for name in profiles.names()? {
        let stored = profiles.load(&name)?;
        let profile = stored.ok_or_else(|| VoiceError::UnknownVoice(name.clone()))?;
        best = closer(best, name, cosine_similarity(embedding, &profile.embedding));
    }
    Ok(best.0)
}
