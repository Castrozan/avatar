use std::collections::VecDeque;

#[derive(Default)]
pub(super) struct Lane {
    pub(super) pending: VecDeque<(u64, Vec<u8>)>,
    pub(super) generation: u64,
    pub(super) playhead: f64,
    pub(super) interrupted: bool,
    pub(super) stopped: bool,
}

pub(super) enum Polled {
    Stopped,
    Chunk((u64, Vec<u8>)),
    Empty,
}

pub(super) fn reserve(lane: &mut Lane, generation: u64, heard_from: (f64, f64)) -> bool {
    let current = lane.generation == generation;
    if current {
        lane.playhead = lane.playhead.max(heard_from.0) + heard_from.1;
    }
    current
}

pub(super) fn polled(lane: &mut Lane) -> Polled {
    match (lane.stopped, lane.pending.pop_front()) {
        (true, _) => Polled::Stopped,
        (false, Some(chunk)) => Polled::Chunk(chunk),
        (false, None) => Polled::Empty,
    }
}
