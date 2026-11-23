#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Meeting,
    Conversation,
    Window,
}

const MEETING_UNITS: [&str; 5] = [
    "avatar-audio",
    "avatar-control-server",
    "avatar-renderer",
    "avatar-browser",
    "avatar-camera",
];
const CONVERSATION_UNITS: [&str; 3] = [
    "avatar-audio",
    "avatar-control-server",
    "avatar-room-microphone",
];
const WINDOW_UNITS: [&str; 3] = ["avatar-control-server", "avatar-renderer", "avatar-browser"];

impl Target {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Meeting => "avatar.target",
            Self::Conversation => "avatar-conversation.target",
            Self::Window => "avatar-window.target",
        }
    }

    #[must_use]
    pub fn systemctl(self, verb: &str) -> Vec<String> {
        ["systemctl", "--user", verb, self.name()]
            .map(str::to_owned)
            .to_vec()
    }

    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        [Self::Meeting, Self::Conversation, Self::Window]
            .into_iter()
            .find(|target| target.name() == name)
    }

    #[must_use]
    pub fn units(self) -> Vec<String> {
        let units: &[&str] = match self {
            Self::Meeting => &MEETING_UNITS,
            Self::Conversation => &CONVERSATION_UNITS,
            Self::Window => &WINDOW_UNITS,
        };
        units.iter().map(|unit| (*unit).to_owned()).collect()
    }
}
