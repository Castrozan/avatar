use std::path::PathBuf;

use avatar_assets::Asset;
use avatar_listening::listener::speaker_identity::{
    Embedder, EmbedderLoader, SpeakerIdentification, VoiceProfiles, configured_identifier,
    voices_directory,
};

const UNAVAILABLE: &str = "this avatar has no speaker embedder";

pub(super) struct NoEmbedding;

impl Embedder for NoEmbedding {
    fn embed(&mut self, _segment: &[u8]) -> Result<Vec<f64>, String> {
        Err(UNAVAILABLE.to_owned())
    }
}

struct UnavailableEmbedder;

impl EmbedderLoader for UnavailableEmbedder {
    type Loaded = NoEmbedding;

    fn load(&mut self, _model_path: &str) -> Result<NoEmbedding, String> {
        Err(UNAVAILABLE.to_owned())
    }
}

fn voice_profiles() -> VoiceProfiles {
    let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_default();
    let state_home = std::env::var("XDG_STATE_HOME").ok();
    VoiceProfiles::new(voices_directory(state_home.as_deref(), &home))
}

pub(super) fn identifier_for(enabled: bool) -> Box<dyn SpeakerIdentification> {
    let model = avatar_assets::resolve(Asset::SpeakerModel)
        .ok()
        .map(|path| path.to_string_lossy().into_owned());
    configured_identifier(enabled, voice_profiles(), model, UnavailableEmbedder)
}
