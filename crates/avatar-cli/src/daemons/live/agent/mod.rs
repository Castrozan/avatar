mod command;
mod quoting;
mod reach;

use avatar_floor::addressee::Addressee;
use avatar_live::boundaries::{AgentTransport, CommandOutput};

pub(crate) struct A2aTransport;

impl AgentTransport for A2aTransport {
    async fn run(&mut self, holder: &Addressee, arguments: &[String]) -> CommandOutput {
        let words = reach::a2a_words(holder.host.as_deref(), arguments);
        command::run_bounded(&words).await
    }
}
