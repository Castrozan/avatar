mod embeddings;
mod location;
mod names;
mod profile_file;
mod profiles;

pub use embeddings::{append_embedding, read_embeddings};
pub use location::voices_directory;
pub use names::validated_name;
pub use profiles::{MATCH_THRESHOLD, VoiceProfile, VoiceProfiles};
