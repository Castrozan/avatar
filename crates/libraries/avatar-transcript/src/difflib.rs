mod comparison;
mod extension;
mod popular;
mod window;

use comparison::{Comparison, collect_blocks};
use window::{MatchingBlock, Window};

#[must_use]
pub fn ratio(first: &[char], second: &[char]) -> f64 {
    let total = first.len() + second.len();
    if total == 0 {
        return 1.0;
    }
    let matches: usize = matching_blocks(first, second)
        .iter()
        .map(|block| block.size)
        .sum();
    2.0 * as_float(matches) / as_float(total)
}

fn as_float(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}

fn matching_blocks(first: &[char], second: &[char]) -> Vec<MatchingBlock> {
    collect_blocks(
        &Comparison::new(first, second),
        Window::whole(first.len(), second.len()),
    )
}
