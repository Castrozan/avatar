use clap::Args;

use super::HEAR_WAIT_SECONDS;
use super::wait::parse_wait;

#[derive(Debug, Clone, PartialEq, Args)]
pub struct SpeechArguments {
    pub text: String,
    #[arg(default_value = "neutral")]
    pub emotion: String,
    #[arg(long, overrides_with = "voice")]
    pub voice: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Args)]
pub struct AskArguments {
    #[command(flatten)]
    pub speech: SpeechArguments,
    #[command(flatten)]
    pub wait: WaitArguments,
}

#[derive(Debug, Clone, PartialEq, Args)]
pub struct WaitArguments {
    #[arg(long, default_value_t = HEAR_WAIT_SECONDS, value_parser = parse_wait, allow_negative_numbers = true, overrides_with = "wait")]
    pub wait: f64,
}
