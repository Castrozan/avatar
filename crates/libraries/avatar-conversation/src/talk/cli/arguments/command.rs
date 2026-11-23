use avatar_listening::conversation_state::ConversationCommand;
use clap::Subcommand;

use super::{AskArguments, OpenArguments, SpeechArguments, WaitArguments};

#[derive(Debug, Clone, PartialEq, Subcommand)]
pub enum TalkCommand {
    #[command(
        about = "Open the conversation",
        infer_long_args = true,
        allow_negative_numbers = true
    )]
    Open(OpenArguments),
    #[command(
        about = "Say text aloud",
        infer_long_args = true,
        allow_negative_numbers = true
    )]
    Say(SpeechArguments),
    #[command(
        about = "Say text aloud and hear the answer",
        infer_long_args = true,
        allow_negative_numbers = true
    )]
    Ask(AskArguments),
    #[command(
        about = "Hear the next utterance",
        infer_long_args = true,
        allow_negative_numbers = true
    )]
    Hear(WaitArguments),
    #[command(flatten)]
    Plain(PlainCommand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum PlainCommand {
    #[command(about = "Follow the room's utterances")]
    Follow,
    #[command(about = "Report the conversation status")]
    Status,
    #[command(about = "Show the avatar window")]
    Show,
    #[command(about = "Hide the avatar window")]
    Hide,
    #[command(about = "Close the conversation")]
    Close,
}

impl PlainCommand {
    const fn name(self) -> &'static str {
        match self {
            Self::Follow => "follow",
            Self::Status => "status",
            Self::Show => "show",
            Self::Hide => "hide",
            Self::Close => "close",
        }
    }

    const fn conversation_command(self) -> ConversationCommand {
        match self {
            Self::Follow => ConversationCommand::Follow,
            Self::Status => ConversationCommand::Status,
            Self::Show => ConversationCommand::Show,
            Self::Hide => ConversationCommand::Hide,
            Self::Close => ConversationCommand::Close,
        }
    }
}

impl TalkCommand {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Open(_) => "open",
            Self::Say(_) => "say",
            Self::Ask(_) => "ask",
            Self::Hear(_) => "hear",
            Self::Plain(plain) => plain.name(),
        }
    }

    #[must_use]
    pub const fn conversation_command(&self) -> ConversationCommand {
        match self {
            Self::Open(_) => ConversationCommand::Open,
            Self::Say(_) => ConversationCommand::Say,
            Self::Ask(_) => ConversationCommand::Ask,
            Self::Hear(_) => ConversationCommand::Hear,
            Self::Plain(plain) => plain.conversation_command(),
        }
    }
}
