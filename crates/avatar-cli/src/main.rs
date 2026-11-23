use std::process::ExitCode;

mod daemons;
mod doctor;
mod holder;
mod mac;
mod start;
mod support;
mod talk;
mod tools;
mod wake;

fn main() -> ExitCode {
    support::cli::run()
}
