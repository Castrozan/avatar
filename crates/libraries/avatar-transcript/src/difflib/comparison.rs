use std::collections::HashSet;

use super::extension::extend_block;
use super::popular::popular_characters;
use super::window::{MatchingBlock, Window, block_ending_at, flanks};

pub(super) struct Comparison<'text> {
    first: &'text [char],
    second: &'text [char],
    popular: HashSet<char>,
}

impl<'text> Comparison<'text> {
    pub(super) fn new(first: &'text [char], second: &'text [char]) -> Self {
        Self {
            first,
            second,
            popular: popular_characters(second),
        }
    }

    fn matches(&self, first_index: usize, second_index: usize) -> bool {
        let character = self.second[second_index];
        self.first[first_index] == character && !self.popular.contains(&character)
    }

    pub(super) fn extends_backward(&self, window: Window, block: MatchingBlock) -> bool {
        block.first_start > window.first_start
            && block.second_start > window.second_start
            && self.first[block.first_start - 1] == self.second[block.second_start - 1]
    }

    pub(super) fn extends_forward(&self, window: Window, block: MatchingBlock) -> bool {
        let first_after = block.first_start + block.size;
        let second_after = block.second_start + block.size;
        first_after < window.first_end
            && second_after < window.second_end
            && self.first[first_after] == self.second[second_after]
    }
}

pub(super) fn collect_blocks(comparison: &Comparison<'_>, window: Window) -> Vec<MatchingBlock> {
    let block = longest_match(comparison, window);
    if block.size == 0 {
        return Vec::new();
    }
    let mut blocks = vec![block];
    blocks.extend(sub_blocks(comparison, window, block));
    blocks
}

fn sub_blocks(
    comparison: &Comparison<'_>,
    window: Window,
    block: MatchingBlock,
) -> Vec<MatchingBlock> {
    flanks(window, block)
        .into_iter()
        .flat_map(|flank| collect_blocks(comparison, flank))
        .collect()
}

fn longest_match(comparison: &Comparison<'_>, window: Window) -> MatchingBlock {
    let mut best = window.empty_block();
    let mut previous = vec![0; window.second_end - window.second_start];
    for first_index in window.first_start..window.first_end {
        previous = scan_row(comparison, first_index, window, &previous, &mut best);
    }
    extend_block(comparison, window, best)
}

fn scan_row(
    comparison: &Comparison<'_>,
    first_index: usize,
    window: Window,
    previous: &[usize],
    best: &mut MatchingBlock,
) -> Vec<usize> {
    let mut current = Vec::with_capacity(previous.len());
    for (offset, second_index) in (window.second_start..window.second_end).enumerate() {
        let carried = offset.checked_sub(1).map_or(0, |prior| previous[prior]);
        let run = usize::from(comparison.matches(first_index, second_index)) * (carried + 1);
        keep_longer(best, block_ending_at(first_index, second_index, run));
        current.push(run);
    }
    current
}

fn keep_longer(best: &mut MatchingBlock, candidate: MatchingBlock) {
    if candidate.size > best.size {
        *best = candidate;
    }
}
