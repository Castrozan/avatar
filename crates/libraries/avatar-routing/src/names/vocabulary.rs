use avatar_assets::ProcessEnvironment;

use crate::text::folding::folded;
use std::collections::HashMap;

pub type Vocabulary = HashMap<String, usize>;

#[must_use]
pub fn frequency_ranks() -> Option<Vocabulary> {
    ranks_from_paths(&avatar_assets::ordinary_word_paths(&ProcessEnvironment))
}

#[must_use]
pub fn ranks_from_paths(paths: &str) -> Option<Vocabulary> {
    let contents = read_all(paths)?;
    Some(
        contents
            .iter()
            .fold(Vocabulary::new(), |ranks, text| merge_ranks(ranks, text)),
    )
}

fn read_all(paths: &str) -> Option<Vec<String>> {
    paths
        .split(':')
        .map(std::fs::read_to_string)
        .collect::<Result<_, _>>()
        .ok()
}

fn merge_ranks(mut ranks: Vocabulary, contents: &str) -> Vocabulary {
    contents
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .enumerate()
        .for_each(|(rank, word)| record_rank(&mut ranks, folded(word), rank));
    ranks
}

fn record_rank(ranks: &mut Vocabulary, word: String, rank: usize) {
    ranks
        .entry(word)
        .and_modify(|best| *best = (*best).min(rank))
        .or_insert(rank);
}
