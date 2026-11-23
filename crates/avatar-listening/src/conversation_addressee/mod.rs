pub mod file_store;
pub mod store;

pub const HERMES_PANE_PREFIX: &str = "hermes:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Addressee {
    pub pane: String,
    pub servant: String,
    pub heard_through: i64,
    pub host: Option<String>,
}

#[must_use]
pub fn hermes_pane(profile: &str) -> String {
    format!("{HERMES_PANE_PREFIX}{profile}")
}

#[must_use]
pub fn holds_without_a_pane(addressee: &Addressee) -> bool {
    addressee.pane.starts_with(HERMES_PANE_PREFIX)
}

#[must_use]
pub fn holder_other_than<'a>(
    current: Option<&'a Addressee>,
    pane: Option<&str>,
) -> Option<&'a Addressee> {
    let holder = current?;
    let speaker = pane?;
    (holder.pane != speaker).then_some(holder)
}

#[must_use]
pub fn addresses(current: Option<&Addressee>, pane: Option<&str>) -> bool {
    holder_other_than(current, pane).is_none()
}
