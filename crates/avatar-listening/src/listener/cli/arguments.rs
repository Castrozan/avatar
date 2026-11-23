use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "avatar-listen", about = "Control the meeting listener")]
pub struct ListenArguments {
    #[command(subcommand)]
    pub command: ListenCommand,
}

#[derive(Debug, Subcommand)]
pub enum ListenCommand {
    Start(StartArguments),
    Stop,
    Status,
    Addressee,
    Read(ReadArguments),
    Follow(FollowArguments),
    Enroll(EnrollArguments),
    Voices,
    Forget(ForgetArguments),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum MeetingSeat {
    Room,
    Remote,
}

#[derive(Debug, Args)]
pub struct StartArguments {
    #[arg(long)]
    pub microphone: Option<String>,
    #[arg(long)]
    pub language: Option<String>,
    #[arg(long, value_enum, default_value_t = MeetingSeat::Room)]
    pub seat: MeetingSeat,
}

#[derive(Debug, Args)]
pub struct ReadArguments {
    #[arg(long, default_value_t = 0)]
    pub since: i64,
    #[arg(long, default_value_t = 0.0)]
    pub wait: f64,
}

#[derive(Debug, Args)]
pub struct FollowArguments {
    #[arg(long, default_value_t = 0)]
    pub since: i64,
    #[arg(long = "match")]
    pub pattern: Option<String>,
}

#[derive(Debug, Args)]
pub struct EnrollArguments {
    pub name: String,
    #[arg(long, allow_hyphen_values = true)]
    pub first: i64,
    #[arg(long, allow_hyphen_values = true)]
    pub last: i64,
}

#[derive(Debug, Args)]
pub struct ForgetArguments {
    pub name: String,
}
