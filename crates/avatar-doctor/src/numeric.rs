#[must_use]
pub(crate) fn count_as_float(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}

#[must_use]
pub(crate) fn whole_number(value: f64) -> Option<i64> {
    format!("{value:.0}").parse().ok()
}

#[must_use]
pub(crate) fn seconds_from_micros(micros: i64) -> Option<f64> {
    format!("{micros}e-6").parse().ok()
}
