use super::scene::Scene;
use crate::claim::Conversation;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Deserialize)]
pub(super) struct ClaimRequest {
    #[serde(flatten)]
    pub(super) scene: Scene,
    pub(super) pane: String,
    pub(super) room: String,
    pub(super) now: f64,
}

#[derive(Deserialize)]
pub(super) struct PaneRequest {
    pub(super) directory: PathBuf,
    pub(super) pane: String,
}

#[derive(Deserialize)]
pub(super) struct MatchRequest {
    pub(super) conversation: Option<Conversation>,
    pub(super) pane: String,
    pub(super) host: String,
}

#[derive(Deserialize)]
pub(super) struct NamesRequest {
    pub(super) conversation: Value,
}
