use super::identifier::{
    DisabledIdentifier, EmbedderLoader, Heard, SpeakerIdentification, SpeakerIdentifier,
};
use super::storage::VoiceProfiles;
use crate::listener::diagnostics::Diagnostics;

impl SpeakerIdentification for Box<dyn SpeakerIdentification> {
    fn identify(&mut self, segment: &[u8], notes: &mut dyn Diagnostics) -> Heard {
        (**self).identify(segment, notes)
    }
}

pub fn configured_identifier<Loader: EmbedderLoader + 'static>(
    enabled: bool,
    profiles: VoiceProfiles,
    model_path: Option<String>,
    loader: Loader,
) -> Box<dyn SpeakerIdentification> {
    if !enabled {
        return Box::new(DisabledIdentifier);
    }
    Box::new(SpeakerIdentifier::new(model_path, profiles, loader))
}
