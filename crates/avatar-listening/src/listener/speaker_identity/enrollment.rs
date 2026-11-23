use std::path::Path;

use super::arithmetic::{mean_vector, unit_vector};
use super::error::VoiceError;
use super::storage::{VoiceProfile, VoiceProfiles, read_embeddings, validated_name};

pub const MINIMUM_ENROLLMENT_EMBEDDINGS: usize = 3;

fn count_of(length: usize) -> i64 {
    i64::try_from(length).unwrap_or(i64::MAX)
}

fn weighted_vectors(
    embeddings: &[Vec<f64>],
    existing: Option<VoiceProfile>,
) -> (Vec<Vec<f64>>, i64) {
    let mut vectors: Vec<Vec<f64>> = embeddings.iter().map(|each| unit_vector(each)).collect();
    let samples = count_of(embeddings.len());
    let width = vectors.first().map_or(0, Vec::len);
    let Some(profile) = existing.filter(|stored| stored.embedding.len() == width) else {
        return (vectors, samples);
    };
    let repeats = usize::try_from(profile.samples).unwrap_or(0);
    let total = samples.saturating_add(profile.samples);
    vectors.extend(std::iter::repeat_n(profile.embedding, repeats));
    (vectors, total)
}

fn chosen_embeddings(path: &Path, range: (i64, i64)) -> Result<Vec<Vec<f64>>, VoiceError> {
    let embeddings = read_embeddings(path, range.0, range.1)?;
    if embeddings.len() >= MINIMUM_ENROLLMENT_EMBEDDINGS {
        return Ok(embeddings);
    }
    Err(VoiceError::TooFewEmbeddings {
        needed: MINIMUM_ENROLLMENT_EMBEDDINGS,
        found: embeddings.len(),
        first: range.0,
        last: range.1,
    })
}

fn require_even(vectors: &[Vec<f64>]) -> Result<(), VoiceError> {
    let width = vectors.first().map_or(0, Vec::len);
    let even = vectors.iter().all(|vector| vector.len() == width);
    even.then_some(()).ok_or(VoiceError::UnevenEmbeddings)
}

pub fn enroll(
    profiles: &VoiceProfiles,
    embeddings_path: &Path,
    name: &str,
    range: (i64, i64),
) -> Result<i64, VoiceError> {
    validated_name(name)?;
    let embeddings = chosen_embeddings(embeddings_path, range)?;
    let (vectors, samples) = weighted_vectors(&embeddings, profiles.load(name)?);
    require_even(&vectors)?;
    profiles.save(name, &mean_vector(&vectors), samples)?;
    Ok(samples)
}
