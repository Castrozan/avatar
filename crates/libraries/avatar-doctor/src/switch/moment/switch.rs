use serde::{Deserialize, Serialize};

use super::moments::Moments;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Switch {
    pub switched_at: f64,
    pub servant: Option<String>,
    pub pane: Option<String>,
    pub sequence: Option<i64>,
    pub delivered: Option<String>,
    pub moments: Moments,
}
