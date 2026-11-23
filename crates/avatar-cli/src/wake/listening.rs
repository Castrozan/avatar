use std::process::ExitCode;

use avatar_adapters::microphone::ParecMicrophone;
use avatar_routing::names::vocabulary::frequency_ranks;
use avatar_wake::addressing::AddressingState;
use avatar_wake::listener::{Boundaries, ConversationSeat, Microphone, WakeListener};

use super::machine::Machine;

fn addressing() -> AddressingState {
    AddressingState {
        vocabulary: frequency_ranks(),
        ..AddressingState::default()
    }
}

pub(super) fn run(machine: &Machine, dry_run: bool) -> ExitCode {
    let mut microphone = ParecMicrophone::new("parec");
    listen(machine, machine, &mut microphone, dry_run)
}

pub(crate) fn listen(
    machine: &Machine,
    seat: &dyn ConversationSeat,
    microphone: &mut dyn Microphone,
    dry_run: bool,
) -> ExitCode {
    let mut listener = WakeListener::new(dry_run, addressing());
    let boundaries = Boundaries {
        effects: machine,
        seat,
    };
    loop {
        listener.advance(boundaries, microphone);
    }
}
