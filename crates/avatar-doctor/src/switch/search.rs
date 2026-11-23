use super::record::{Logged, Record};

#[must_use]
pub(crate) fn latest(
    logged: &[Logged],
    start: usize,
    end: usize,
    wanted: impl Fn(&Record) -> bool,
) -> Option<&Logged> {
    (start..end)
        .rev()
        .filter_map(|position| logged.get(position))
        .find(|item| wanted(&item.record))
}

#[must_use]
pub(crate) fn first_after(
    logged: &[Logged],
    index: usize,
    until: f64,
    wanted: impl Fn(&Record) -> bool,
) -> Option<&Logged> {
    logged
        .iter()
        .skip(index + 1)
        .take_while(|item| item.at < until)
        .find(|item| wanted(&item.record))
}
