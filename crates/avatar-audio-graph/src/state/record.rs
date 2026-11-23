use serde::{Deserialize, Serialize};

use crate::pulse::module_record::ModuleRecord;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioState {
    #[serde(default)]
    pub modules: Vec<ModuleRecord>,
    #[serde(default)]
    pub ready: bool,
    #[serde(default)]
    pub microphone: Option<MicrophoneRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicrophoneRecord {
    pub source: String,
    #[serde(flatten)]
    pub module: ModuleRecord,
}
