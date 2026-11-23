use avatar_audio_graph::state::record::AudioState;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct ScenarioFixture {
    pub outputs: Vec<String>,
    pub scenarios: Vec<ScenarioRecord>,
}

#[derive(Deserialize)]
pub(crate) struct ScenarioRecord {
    pub name: String,
    pub steps: Vec<StepRecord>,
}

#[derive(Deserialize)]
pub(crate) struct StepRecord {
    pub step: StepSpec,
    pub calls: Vec<CallRecord>,
    pub result: ResultRecord,
    pub state: AudioState,
    pub file_exists: bool,
}

#[derive(Deserialize)]
pub(crate) struct StepSpec {
    #[serde(rename = "do")]
    pub action: String,
    pub source: Option<String>,
    pub channel: Option<String>,
    pub value: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct CallRecord {
    pub argv: Vec<String>,
    pub output: Option<usize>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ResultRecord {
    pub ok: Option<bool>,
    pub error: Option<String>,
}
