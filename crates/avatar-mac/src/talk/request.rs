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
}
