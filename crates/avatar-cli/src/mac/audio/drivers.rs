use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use avatar_seat::edges::{EdgeFailure, SystemProcesses};
use avatar_seat::meetings::StandardOutput;
use avatar_seat::meetings::drivers::{
    DRIVER_DIRECTORY, DriverEdges, DriverPlan, FileSystemDrivers, install_call_audio_drivers,
};
use clap::Args;

#[derive(Args)]
pub(crate) struct DriverOptions {
    #[arg(help = "The packaged driver bundles to install when they are missing or differ")]
    packaged: Vec<PathBuf>,
}

fn exit_of(outcome: Result<(), EdgeFailure>) -> ExitCode {
    let Err(failure) = outcome else {
        return ExitCode::SUCCESS;
    };
    let _ = writeln!(io::stderr(), "{failure}");
    ExitCode::FAILURE
}

pub(super) fn run(options: &DriverOptions) -> ExitCode {
    let directory = PathBuf::from(DRIVER_DIRECTORY);
    let plan = DriverPlan {
        directory,
        packaged: options.packaged.clone(),
    };
    let edges = DriverEdges::new(&SystemProcesses, &FileSystemDrivers, &StandardOutput);
    exit_of(install_call_audio_drivers(&edges, &plan))
}
