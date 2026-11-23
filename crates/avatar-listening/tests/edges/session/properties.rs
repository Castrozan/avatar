use super::fake_units::FakeUnits;
use avatar_listening::conversation_state::ListenerPurpose;
use avatar_listening::listener::session::{SessionDirectory, SessionRequest, begin_listening};
use proptest::prelude::*;
use serde_json::{Map, Value, json};

fn failure(error: impl std::fmt::Display) -> String {
    format!("{error}")
}

fn request_of(settings: &Value) -> SessionRequest {
    let settings = settings.as_object().cloned().unwrap_or_default();
    SessionRequest {
        purpose: ListenerPurpose::Meeting,
        sources: Map::new(),
        settings,
    }
}

fn written_configuration(settings: &Value) -> Result<Value, String> {
    let root = tempfile::tempdir().map_err(failure)?;
    let directory = SessionDirectory::new(root.path().join("state"));
    let mut units = FakeUnits::with_active(Vec::new());
    begin_listening(&directory, &mut units, &request_of(settings)).map_err(failure)?;
    let bytes = std::fs::read(root.path().join("state/listener.json")).map_err(failure)?;
    serde_json::from_slice(&bytes).map_err(failure)
}

proptest! {
    #[test]
    fn the_configuration_is_purpose_sources_and_every_setting(language in "[a-z]{0,4}", identify in any::<bool>()) {
        let settings = json!({"language": language, "identify_speakers": identify});
        let expected = json!({"purpose": "meeting", "sources": {}, "language": language, "identify_speakers": identify});
        prop_assert_eq!(written_configuration(&settings), Ok(expected));
    }
}
