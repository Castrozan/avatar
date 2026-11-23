use super::storage::VoiceProfiles;
use crate::listener::diagnostics::Diagnostics;

pub const MODEL_ENVIRONMENT_VARIABLE: &str = "AVATAR_SPEAKER_MODEL";

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Heard {
    pub speaker: Option<String>,
    pub embedding: Vec<f64>,
}

pub trait Embedder {
    fn embed(&mut self, segment: &[u8]) -> Result<Vec<f64>, String>;
}

pub trait EmbedderLoader {
    type Loaded: Embedder;
    fn load(&mut self, model_path: &str) -> Result<Self::Loaded, String>;
}

pub trait SpeakerIdentification {
    fn identify(&mut self, segment: &[u8], notes: &mut dyn Diagnostics) -> Heard;
}

pub struct DisabledIdentifier;

impl SpeakerIdentification for DisabledIdentifier {
    fn identify(&mut self, _segment: &[u8], _notes: &mut dyn Diagnostics) -> Heard {
        Heard::default()
    }
}

enum Extractor<Loaded> {
    NotTried,
    Ready(Loaded),
    Unavailable,
}

pub struct SpeakerIdentifier<Loader: EmbedderLoader> {
    model_path: Option<String>,
    profiles: VoiceProfiles,
    loader: Loader,
    extractor: Extractor<Loader::Loaded>,
}

fn announce_off<Loaded>(
    loaded: Result<Loaded, String>,
    notes: &mut dyn Diagnostics,
) -> Extractor<Loaded> {
    match loaded {
        Ok(extractor) => Extractor::Ready(extractor),
        Err(error) => {
            notes.note(&format!("Speaker identity is off: {error}"));
            Extractor::Unavailable
        }
    }
}

fn announce_failure(failure: Result<Heard, String>, notes: &mut dyn Diagnostics) -> Heard {
    failure.unwrap_or_else(|error| {
        notes.note(&format!("Speaker identity failed for a segment: {error}"));
        Heard::default()
    })
}

impl<Loader: EmbedderLoader> SpeakerIdentifier<Loader> {
    pub fn new(model_path: Option<String>, profiles: VoiceProfiles, loader: Loader) -> Self {
        Self {
            model_path,
            profiles,
            loader,
            extractor: Extractor::NotTried,
        }
    }

    fn load_extractor(&mut self, notes: &mut dyn Diagnostics) {
        let path = self.model_path.as_deref().filter(|path| !path.is_empty());
        let missing = format!("{MODEL_ENVIRONMENT_VARIABLE} is not set");
        let loaded = path.map_or(Err(missing), |model| self.loader.load(model));
        self.extractor = announce_off(loaded, notes);
    }

    fn try_identify(&mut self, segment: &[u8]) -> Result<Heard, String> {
        identify_with(&mut self.extractor, &self.profiles, segment)
    }
}

fn identify_with<Loaded: Embedder>(
    extractor: &mut Extractor<Loaded>,
    profiles: &VoiceProfiles,
    segment: &[u8],
) -> Result<Heard, String> {
    let Extractor::Ready(embedder) = extractor else {
        return Ok(Heard::default());
    };
    let embedding = embedder.embed(segment)?;
    let speaker = profiles
        .best_match(&embedding)
        .map_err(|error| error.to_string())?;
    Ok(Heard { speaker, embedding })
}

impl<Loader: EmbedderLoader> SpeakerIdentification for SpeakerIdentifier<Loader> {
    fn identify(&mut self, segment: &[u8], notes: &mut dyn Diagnostics) -> Heard {
        matches!(self.extractor, Extractor::NotTried).then(|| self.load_extractor(notes));
        announce_failure(self.try_identify(segment), notes)
    }
}
