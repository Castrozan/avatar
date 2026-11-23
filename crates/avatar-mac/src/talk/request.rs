use avatar_conversation::talk::cli::arguments::AgentSpec;
use avatar_listening::conversation_state::ConversationCommand;

#[derive(Debug, Clone, PartialEq)]
pub struct TalkRequest {
    pub command: ConversationCommand,
    pub language: Option<String>,
    pub character: Option<String>,
    pub text: String,
    pub emotion: String,
    pub voice: Option<String>,
    pub wait: f64,
    pub agent: Option<AgentSpec>,
}

impl TalkRequest {
    #[must_use]
    pub const fn new(command: ConversationCommand) -> Self {
        Self {
            command,
            language: None,
            character: None,
            text: String::new(),
            emotion: String::new(),
            voice: None,
            wait: 0.0,
            agent: None,
        }
    }
}
