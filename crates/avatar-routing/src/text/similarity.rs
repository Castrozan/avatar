use std::cmp::Reverse;
use std::collections::HashSet;

use crate::text::similarity_popular::popular_characters;

struct Span {
    left: std::ops::Range<usize>,
    right: std::ops::Range<usize>,
}

struct Sequences {
    left: Vec<char>,
    right: Vec<char>,
    popular: HashSet<char>,
}

#[must_use]
pub fn ratio(left: &str, right: &str) -> f64 {
    let sequences = sequences_of(left, right);
    let whole = span(0..sequences.left.len(), 0..sequences.right.len());
    let total = whole.left.len() + whole.right.len();
    match total {
        0 => 1.0,
        _ => 2.0 * as_float(matching_total(&sequences, &whole)) / as_float(total),
    }
}

fn sequences_of(left: &str, right: &str) -> Sequences {
    let right: Vec<char> = right.chars().collect();
    let popular = popular_characters(&right);
    let left = left.chars().collect();
    Sequences {
        left,
        right,
        popular,
    }
}

fn span(left: std::ops::Range<usize>, right: std::ops::Range<usize>) -> Span {
    Span { left, right }
}

fn matching_total(sequences: &Sequences, region: &Span) -> usize {
    let block = longest_match(sequences, region);
    match block.left.len() {
        0 => 0,
        size => size + remainder_total(sequences, region, &block),
    }
}

fn remainder_total(sequences: &Sequences, region: &Span, block: &Span) -> usize {
    let left = region.left.start..block.left.start;
    let right = region.right.start..block.right.start;
    let before = span(left, right);
    let left = block.left.end..region.left.end;
    let right = block.right.end..region.right.end;
    matching_total(sequences, &before) + matching_total(sequences, &span(left, right))
}

fn longest_match(sequences: &Sequences, region: &Span) -> Span {
    let Span { left, right } = region;
    let origin = span(left.start..left.start, right.start..right.start);
    let best = (0..left.len() * right.len())
        .map(|index| run_at(sequences, region, index))
        .min_by_key(preference);
    extend(sequences, region, &best.unwrap_or(origin))
}

fn preference(Span { left, right }: &Span) -> (Reverse<usize>, usize, usize) {
    (Reverse(left.len()), left.start, right.start)
}

fn run_at(sequences: &Sequences, region: &Span, index: usize) -> Span {
    let left_index = region.left.start + index / region.right.len();
    let right_index = region.right.start + index % region.right.len();
    let size = sequences.left[left_index..region.left.end]
        .iter()
        .zip(&sequences.right[right_index..region.right.end])
        .take_while(|pair| pair.0 == pair.1 && !sequences.popular.contains(pair.0))
        .count();
    let left = left_index..left_index + size;
    span(left, right_index..right_index + size)
}

fn extend(sequences: &Sequences, region: &Span, block: &Span) -> Span {
    let backward = backward_extension(sequences, region, block);
    let forward = forward_extension(sequences, region, block);
    let left = block.left.start - backward..block.left.end + forward;
    let right = block.right.start - backward..block.right.end + forward;
    span(left, right)
}

fn backward_extension(sequences: &Sequences, region: &Span, block: &Span) -> usize {
    let left = &sequences.left[region.left.start..block.left.start];
    let right = &sequences.right[region.right.start..block.right.start];
    agreeing_run(left.iter().rev().copied(), right.iter().rev().copied())
}

fn forward_extension(sequences: &Sequences, region: &Span, block: &Span) -> usize {
    let left = &sequences.left[block.left.end..region.left.end];
    let right = &sequences.right[block.right.end..region.right.end];
    agreeing_run(left.iter().copied(), right.iter().copied())
}

fn agreeing_run(left: impl Iterator<Item = char>, right: impl Iterator<Item = char>) -> usize {
    left.zip(right).take_while(|pair| pair.0 == pair.1).count()
}

fn as_float(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}
