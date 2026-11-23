use std::io::{self, Write};
use std::process::ExitCode;

use avatar_adapters::commands::Endpoints;
use avatar_floor::quiet::Seat;
use avatar_seat::edges::{HttpProbe, SystemClock, SystemProcesses};
use avatar_seat::meetings::StandardOutput;
use avatar_seat::meetings::readiness::SystemReadiness;
use avatar_seat::meetings::start::{StartEdges, start};
use clap::{Args, ValueEnum};

mod preparation;

use preparation::ConversationSeatPreparation;

#[derive(Clone, Copy, ValueEnum)]
enum MeetingSeat {
    Room,
    Remote,
}

#[derive(Args)]
pub(crate) struct StartOptions {
    #[arg(long, value_enum, default_value_t = MeetingSeat::Room, help = "Where the call's people sit")]
    seat: MeetingSeat,
    #[arg(
        long,
        help = "Start fresh virtual meeting devices without physical capture; refuse an existing runtime"
    )]
    virtual_only: bool,
}

const fn seat_of(choice: MeetingSeat) -> Seat {
    match choice {
        MeetingSeat::Room => Seat::Room,
        MeetingSeat::Remote => Seat::Remote,
    }
}

fn readiness<'edges>(
    processes: &'edges SystemProcesses,
    health: &'edges HttpProbe,
) -> SystemReadiness<'edges> {
    SystemReadiness {
        processes,
        control_health: health,
    }
}

fn edges<'edges>(
    processes: &'edges SystemProcesses,
    readiness: &'edges SystemReadiness<'edges>,
    clock: &'edges SystemClock,
) -> StartEdges<'edges> {
    StartEdges {
        processes,
        clock,
        readiness,
        seats: &ConversationSeatPreparation,
        output: &StandardOutput,
    }
}

pub(crate) fn run(options: &StartOptions) -> ExitCode {
    let processes = SystemProcesses;
    let health = HttpProbe {
        port: Endpoints::default().control_port,
    };
    let readiness = readiness(&processes, &health);
    let clock = SystemClock::new();
    let edges = edges(&processes, &readiness, &clock);
    finish(start(&edges, seat_of(options.seat), options.virtual_only))
}

fn finish(outcome: Result<(), impl std::fmt::Display>) -> ExitCode {
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            let _ = writeln!(io::stderr(), "{failure}");
            ExitCode::FAILURE
        }
    }
}
