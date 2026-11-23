use avatar_listening::listener::diagnostics::Diagnostics;
use avatar_listening::listener::speaker_identity::{
    DisabledIdentifier, Embedder, EmbedderLoader, Heard, SpeakerIdentification, SpeakerIdentifier,
    VoiceProfiles,
};

#[derive(Default)]
struct Notes(Vec<String>);

impl Diagnostics for Notes {
    fn note(&mut self, line: &str) {
        self.0.push(line.to_owned());
    }
}

struct FixedEmbedder(Vec<f64>);

impl Embedder for FixedEmbedder {
    fn embed(&mut self, _segment: &[u8]) -> Result<Vec<f64>, String> {
        Ok(self.0.clone())
    }
}

#[derive(Default)]
struct Loader {
    attempts: usize,
    failure: Option<String>,
}

impl EmbedderLoader for Loader {
    type Loaded = FixedEmbedder;

    fn load(&mut self, _model_path: &str) -> Result<FixedEmbedder, String> {
        self.attempts += 1;
        self.failure
            .clone()
            .map_or_else(|| Ok(FixedEmbedder(vec![1.0, 0.0])), Err)
    }
}

#[test]
fn a_listener_without_identity_names_no_one_and_keeps_no_embedding() {
    let heard = DisabledIdentifier.identify(&[0, 0], &mut Notes::default());
    assert_eq!(heard, Heard::default());
}

type Outcome = Result<(), Box<dyn std::error::Error>>;

fn identifier_over(
    model: Option<&str>,
    loader: Loader,
) -> Result<(SpeakerIdentifier<Loader>, tempfile::TempDir), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let profiles = VoiceProfiles::new(root.path());
    profiles.save("ana", &[1.0, 0.0], 3)?;
    let identifier = SpeakerIdentifier::new(model.map(str::to_owned), profiles, loader);
    Ok((identifier, root))
}

#[test]
fn a_segment_is_named_by_its_closest_enrolled_voice() -> Outcome {
    let (mut identifier, _root) = identifier_over(Some("model.onnx"), Loader::default())?;
    let heard = identifier.identify(b"audio", &mut Notes::default());
    let expected = Heard {
        speaker: Some("ana".into()),
        embedding: vec![1.0, 0.0],
    };
    assert_eq!(heard, expected);
    Ok(())
}

#[test]
fn a_model_that_fails_to_load_leaves_speakers_unknown_and_is_tried_once() -> Outcome {
    let loader = Loader {
        attempts: 0,
        failure: Some("no model".into()),
    };
    let (mut identifier, _root) = identifier_over(Some("model.onnx"), loader)?;
    let mut notes = Notes::default();
    assert_eq!(identifier.identify(b"audio", &mut notes), Heard::default());
    assert_eq!(identifier.identify(b"audio", &mut notes), Heard::default());
    assert_eq!(notes.0, ["Speaker identity is off: no model"]);
    Ok(())
}

#[test]
fn a_missing_model_path_leaves_speakers_unknown() -> Outcome {
    let (mut identifier, _root) = identifier_over(None, Loader::default())?;
    let mut notes = Notes::default();
    assert_eq!(identifier.identify(b"audio", &mut notes), Heard::default());
    assert_eq!(
        notes.0,
        ["Speaker identity is off: AVATAR_SPEAKER_MODEL is not set"]
    );
    Ok(())
}
