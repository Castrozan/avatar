mod error;
mod generated;
mod generator;
mod request;

pub use error::SynthesisError;
pub use generated::{GeneratedSpeech, SpeechSource, SynthesisResult};
pub use generator::SpeechGenerator;
pub use request::{SpeechRequest, non_empty, voice_or};
