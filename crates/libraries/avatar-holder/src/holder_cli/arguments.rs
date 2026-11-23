use clap::Parser;

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(name = "avatar-voice-holder")]
pub struct HolderArguments {
    #[arg(long)]
    pub hermes_profile: String,
    #[arg(long)]
    pub character: Option<String>,
}

pub fn parse_arguments(arguments: &[String]) -> Result<HolderArguments, clap::Error> {
    let program = std::iter::once("avatar-voice-holder");
    HolderArguments::try_parse_from(program.chain(arguments.iter().map(String::as_str)))
}
