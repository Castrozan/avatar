pub struct RoomVoiceState {
    speaking_voice: String,
}

impl RoomVoiceState {
    #[must_use]
    pub fn new(speaking_voice: &str) -> Self {
        Self {
            speaking_voice: speaking_voice.to_owned(),
        }
    }

    #[must_use]
    pub fn speaking_voice(&self) -> &str {
        &self.speaking_voice
    }

    #[must_use]
    pub fn wants_change(&self, wanted: Option<&str>) -> bool {
        wanted.is_some_and(|voice| !voice.is_empty() && voice != self.speaking_voice)
    }

    pub fn change_to(&mut self, speaking_voice: &str) {
        speaking_voice.clone_into(&mut self.speaking_voice);
    }
}
