#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Window {
    pub(super) first_start: usize,
    pub(super) first_end: usize,
    pub(super) second_start: usize,
    pub(super) second_end: usize,
}

impl Window {
    pub(super) fn whole(first_length: usize, second_length: usize) -> Self {
        Self {
            first_start: 0,
            first_end: first_length,
            second_start: 0,
            second_end: second_length,
        }
    }

    pub(super) fn empty_block(self) -> MatchingBlock {
        MatchingBlock {
            first_start: self.first_start,
            second_start: self.second_start,
            size: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MatchingBlock {
    pub(super) first_start: usize,
    pub(super) second_start: usize,
    pub(super) size: usize,
}

pub(super) fn block_ending_at(
    first_index: usize,
    second_index: usize,
    run: usize,
) -> MatchingBlock {
    MatchingBlock {
        first_start: first_index + 1 - run,
        second_start: second_index + 1 - run,
        size: run,
    }
}

pub(super) fn flanks(window: Window, block: MatchingBlock) -> Vec<Window> {
    [leading_flank(window, block), trailing_flank(window, block)]
        .into_iter()
        .flatten()
        .collect()
}

fn leading_flank(window: Window, block: MatchingBlock) -> Option<Window> {
    let fits = window.first_start < block.first_start && window.second_start < block.second_start;
    fits.then_some(Window {
        first_start: window.first_start,
        first_end: block.first_start,
        second_start: window.second_start,
        second_end: block.second_start,
    })
}

fn trailing_flank(window: Window, block: MatchingBlock) -> Option<Window> {
    let first_after = block.first_start + block.size;
    let second_after = block.second_start + block.size;
    let fits = first_after < window.first_end && second_after < window.second_end;
    fits.then_some(Window {
        first_start: first_after,
        first_end: window.first_end,
        second_start: second_after,
        second_end: window.second_end,
    })
}
