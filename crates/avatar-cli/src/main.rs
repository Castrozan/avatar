use std::process::ExitCode;

mod cli;
mod doctor;
mod holder;
mod mac;
mod start;
mod support;
mod talk;
mod tools;
mod wake;

fn main() -> ExitCode {
    cli::run()
}
