use super::comparison::Comparison;
use super::window::{MatchingBlock, Window};

pub(super) fn extend_block(
    comparison: &Comparison<'_>,
    window: Window,
    block: MatchingBlock,
) -> MatchingBlock {
    let backward = extend_backward(comparison, window, block);
    extend_forward(comparison, window, backward)
}

fn extend_backward(
    comparison: &Comparison<'_>,
    window: Window,
    mut block: MatchingBlock,
) -> MatchingBlock {
    while comparison.extends_backward(window, block) {
        block.first_start -= 1;
        block.second_start -= 1;
        block.size += 1;
    }
    block
}

fn extend_forward(
    comparison: &Comparison<'_>,
    window: Window,
    mut block: MatchingBlock,
) -> MatchingBlock {
    while comparison.extends_forward(window, block) {
        block.size += 1;
    }
    block
}
