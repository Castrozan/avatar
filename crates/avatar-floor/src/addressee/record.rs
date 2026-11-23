use serde::{Deserialize, Serialize};

const HERMES_PANE_PREFIX: &str = "hermes:";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Addressee {
    pub pane: String,
    pub servant: String,
    pub heard_through: i64,
    #[serde(default)]
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
