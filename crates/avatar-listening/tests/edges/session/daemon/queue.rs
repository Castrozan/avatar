use std::sync::Arc;

use super::fakes::{Events, Notes};
use avatar_listening::listener::capture::{HeardMoments, SegmentDesk};
use avatar_listening::listener::daemon::{PENDING_SEGMENT_LIMIT, SegmentSource, segment_queue};
use avatar_listening::listener::speaker_identity::{
    DisabledIdentifier, SpeakerIdentification, VoiceProfiles, configured_identifier,
};

const HEARD: HeardMoments = HeardMoments {
    speech_ended: 1.0,
    segment_closed: 2.0,
};

#[test]
fn a_full_queue_refuses_the_segment_and_says_transcription_is_behind() {
    let (mut desk, mut receiver) = segment_queue(Arc::new(Events::default()), Notes::default());
    let accepted = (0..PENDING_SEGMENT_LIMIT).all(|_| desk.enqueue("human", vec![1], true, HEARD));
    assert!(accepted);
    assert!(!desk.enqueue("meeting", vec![2], false, HEARD));
    assert_eq!(
        receiver.next(0.01).map(|segment| segment.source),
        Some("human".to_owned())
    );
}

#[test]
fn an_empty_queue_times_out_without_a_segment() {
    let (_desk, mut receiver) = segment_queue(Arc::new(Events::default()), Notes::default());
    assert_eq!(receiver.next(0.01), None);
}

#[test]
fn a_disabled_configuration_names_no_one_and_keeps_no_embedding()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let profiles = VoiceProfiles::new(directory.path());
    let mut identifier = configured_identifier(false, profiles, Some("model".into()), Loader);
    assert_eq!(
        identifier.identify(&[0, 0], &mut Notes::default()),
        DisabledIdentifier.identify(&[], &mut Notes::default())
    );
    Ok(())
}

struct Loader;

impl avatar_listening::listener::speaker_identity::EmbedderLoader for Loader {
    type Loaded = Never;

    fn load(&mut self, _model_path: &str) -> Result<Never, String> {
        Err("never loaded".into())
    }
}

struct Never;

impl avatar_listening::listener::speaker_identity::Embedder for Never {
    fn embed(&mut self, _segment: &[u8]) -> Result<Vec<f64>, String> {
        Err("never".into())
    }
}
