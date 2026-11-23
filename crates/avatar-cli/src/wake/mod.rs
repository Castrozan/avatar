use std::io;
use std::process::ExitCode;

use clap::Args;

mod listening;
pub(crate) mod machine;
mod query;

use machine::Machine;

#[derive(Args)]
pub(crate) struct WakeOptions {
    #[arg(
        long,
        help = "Report what would be delivered without delivering or handing over"
    )]
    dry_run: bool,
    #[arg(long, help = "Print this machine's live sessions as JSON")]
    sessions: bool,
    #[arg(long, help = "Print the conversation this machine is holding as JSON")]
    conversation: bool,
    #[arg(
        long,
        help = "Claim the wake for this pane on behalf of --room; exit 0 if won"
    )]
    claim: Option<String>,
    #[arg(long, help = "The room a --claim is made for")]
    room: Option<String>,
    #[arg(
        long,
        help = "Print whether the room spoke within this many seconds as JSON"
    )]
    heard_within: Option<f64>,
}

enum Mode<'options> {
    Conversation,
    Claim(&'options str),
    Sessions,
    Heard(f64),
    Listen,
}

impl WakeOptions {
    fn mode(&self) -> Mode<'_> {
        let claim = self.claim.as_deref().filter(|pane| !pane.is_empty());
        match (self.conversation, claim, self.sessions, self.heard_within) {
            (true, ..) => Mode::Conversation,
            (_, Some(pane), ..) => Mode::Claim(pane),
            (_, _, true, _) => Mode::Sessions,
            (_, _, _, Some(seconds)) => Mode::Heard(seconds),
            _ => Mode::Listen,
        }
    }
}

pub(crate) fn run(options: &WakeOptions) -> io::Result<ExitCode> {
    let machine = Machine::on_this_machine()?;
    match options.mode() {
        Mode::Conversation => query::conversation(&machine),
        Mode::Claim(pane) => query::claim(&machine, pane, options.room.as_deref()),
        Mode::Sessions => query::sessions(&machine),
        Mode::Heard(seconds) => query::heard(&machine, seconds),
        Mode::Listen => Ok(listening::run(&machine, options.dry_run)),
    }
}
