mod arithmetic;
mod configured;
mod enrollment;
mod error;
mod identifier;
mod storage;

pub use arithmetic::{cosine_similarity, mean_vector, python_hypot, python_sum, unit_vector};
pub use configured::configured_identifier;
pub use enrollment::{MINIMUM_ENROLLMENT_EMBEDDINGS, enroll};
pub use error::VoiceError;
pub use identifier::{
    DisabledIdentifier, Embedder, EmbedderLoader, Heard, MODEL_ENVIRONMENT_VARIABLE,
    SpeakerIdentification, SpeakerIdentifier,
};
pub use storage::{
    MATCH_THRESHOLD, VoiceProfile, VoiceProfiles, append_embedding, read_embeddings,
    validated_name, voices_directory,
};
