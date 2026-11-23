use serde::Serialize;

use super::scene::Source;

#[derive(Serialize)]
pub(super) struct Selection {
    pub(super) name: &'static str,
    pub(super) current_scene: &'static str,
    pub(super) current_program_scene: &'static str,
}

#[derive(Serialize)]
pub(super) struct SceneName {
    pub(super) name: &'static str,
}

#[derive(Serialize, Default)]
pub(super) struct Unused {
    pub(super) groups: Vec<u8>,
    pub(super) transitions: Vec<u8>,
    pub(super) quick_transitions: Vec<u8>,
}

#[derive(Serialize)]
pub(super) struct StartScriptSettings {
    pub(super) report_path: String,
}

#[derive(Serialize)]
pub(super) struct StartScript {
    pub(super) path: String,
    pub(super) settings: StartScriptSettings,
}

#[derive(Serialize)]
pub(super) struct Modules {
    #[serde(rename = "scripts-tool")]
    pub(super) scripts_tool: Vec<StartScript>,
}

#[derive(Serialize)]
pub(super) struct SceneCollection {
    #[serde(flatten)]
    pub(super) selection: Selection,
    pub(super) scene_order: Vec<SceneName>,
    pub(super) sources: Vec<Source>,
    #[serde(flatten)]
    pub(super) unused: Unused,
    pub(super) modules: Modules,
}
