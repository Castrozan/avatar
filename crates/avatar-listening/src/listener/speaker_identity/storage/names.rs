use crate::listener::speaker_identity::error::VoiceError;

pub fn validated_name(name: &str) -> Result<&str, VoiceError> {
    let allowed = |character: char| character.is_ascii_alphanumeric() || "_-".contains(character);
    if !name.is_empty() && name.chars().all(allowed) {
        return Ok(name);
    }
    Err(VoiceError::InvalidName)
}
