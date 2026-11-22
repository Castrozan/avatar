use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Session {
    pub servant: String,
    pub pane: String,
    #[serde(default)]
    pub host: Option<String>,
}
