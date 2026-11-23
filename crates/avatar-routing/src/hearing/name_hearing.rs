use std::collections::HashSet;

use crate::hearing::japanese_similarity::japanese_similarity;
use crate::hearing::name_similarity::{joins_fragments, sounded_similarity};
use crate::names::vocabulary::Vocabulary;
use crate::text::sound::sound_key;

const SPELLED_ALIKE: f64 = 2.0;
type SimilarityOf = fn(&[String], &str, Option<&Vocabulary>) -> Option<f64>;
const SIMILARITY_FUNCTIONS: [SimilarityOf; 2] = [heard_similarity, japanese_similarity];

struct Hearing {
    similarity: f64,
    name: String,
    size: usize,
}

#[must_use]
pub fn heard_name(
    windows: &[Vec<String>],
    names: &[String],
    vocabulary: Option<&Vocabulary>,
) -> Option<(String, usize)> {
    SIMILARITY_FUNCTIONS
        .iter()
        .map(|similarity_of| hearings(*similarity_of, windows, names, vocabulary))
        .find(|heard| !heard.is_empty())
        .and_then(|heard| leading_name(&leaders_of(&heard)))
}

fn hearings(
    similarity_of: SimilarityOf,
    windows: &[Vec<String>],
    names: &[String],
    vocabulary: Option<&Vocabulary>,
) -> Vec<Hearing> {
    windows
        .iter()
        .flat_map(|words| hearings_for_window(similarity_of, words, names, vocabulary))
        .collect()
}

fn hearings_for_window(
    similarity_of: SimilarityOf,
    words: &[String],
    names: &[String],
    vocabulary: Option<&Vocabulary>,
) -> impl Iterator<Item = Hearing> {
    names
        .iter()
        .filter_map(move |name| hearing_of(similarity_of, words, name, vocabulary))
}

fn leaders_of(heard: &[Hearing]) -> Vec<&Hearing> {
    let clearest = clearest_similarity(heard);
    heard
        .iter()
        .filter(|hearing| hearing.similarity.to_bits() == clearest.to_bits())
        .collect()
}

fn clearest_similarity(heard: &[Hearing]) -> f64 {
    heard
        .iter()
        .map(|hearing| hearing.similarity)
        .fold(f64::NEG_INFINITY, f64::max)
}

fn leading_name(leaders: &[&Hearing]) -> Option<(String, usize)> {
    let distinct: HashSet<&str> = leaders.iter().map(|leader| leader.name.as_str()).collect();
    match distinct.len() {
        1 => leaders
            .iter()
            .min_by_key(|leader| leader.size)
            .map(|leader| (leader.name.clone(), leader.size)),
        _ => None,
    }
}

fn hearing_of(
    similarity_of: SimilarityOf,
    words: &[String],
    name: &str,
    vocabulary: Option<&Vocabulary>,
) -> Option<Hearing> {
    let similarity = similarity_of(words, name, vocabulary)?;
    Some(Hearing {
        similarity,
        name: name.to_owned(),
        size: words.len(),
    })
}

fn heard_similarity(words: &[String], name: &str, vocabulary: Option<&Vocabulary>) -> Option<f64> {
    if joins_fragments(words) {
        return None;
    }
    let heard = words.concat();
    if heard == name {
        return Some(SPELLED_ALIKE);
    }
    sounded_similarity(words, &sound_key(&heard), &sound_key(name), vocabulary)
}
