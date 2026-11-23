use std::fs;
use std::io::{self, Write};
use std::process::ExitCode;

use avatar_mac::client::{hush_and_leave, play_recording};
use avatar_mac::edge::client::SeatClient;
use avatar_mac::edge::stage::Afplay;
use avatar_mac::edge::terminate::until_terminated;
use avatar_mac::failure::{Failure, FailureKind};
use clap::Args;
use tokio::runtime::Builder;
use tokio::task::JoinError;

#[derive(Args)]
pub(crate) struct PlayOptions {
    #[arg(help = "The recording to play")]
    recording: String,
}

fn read_recording(recording: &str) -> Result<Vec<u8>, Failure> {
    fs::read(recording).map_err(|error| Failure::new(FailureKind::Os, &error.to_string()))
}

fn play(recording: &str) -> Result<(), Failure> {
    let mut endpoint = SeatClient::on_this_mac()?;
    play_recording(
        &mut endpoint,
        &mut Afplay::system(),
        recording,
        read_recording,
    )
}

fn leave_quietly() -> ExitCode {
    let code = SeatClient::on_this_mac().map_or(1, |mut endpoint| hush_and_leave(&mut endpoint));
    ExitCode::from(u8::try_from(code).unwrap_or(1))
}

fn report(failure: &Failure) -> ExitCode {
    let _ = writeln!(io::stderr(), "{}", failure.message);
    ExitCode::FAILURE
}

type Played = io::Result<Option<Result<Result<(), Failure>, JoinError>>>;

fn finished(outcome: Played) -> ExitCode {
    match outcome {
        Ok(None) => leave_quietly(),
        Ok(Some(Ok(Ok(())))) => ExitCode::SUCCESS,
        Ok(Some(Ok(Err(failure)))) => report(&failure),
        _ => ExitCode::FAILURE,
    }
}

pub(super) fn run(options: &PlayOptions) -> ExitCode {
    let recording = options.recording.clone();
    let Ok(runtime) = Builder::new_multi_thread().enable_all().build() else {
        return ExitCode::FAILURE;
    };
    let playing = tokio::task::spawn_blocking(move || play(&recording));
    finished(runtime.block_on(until_terminated(playing)))
}
