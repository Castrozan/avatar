use avatar_routing::protocol::json_lines::run_json_lines;
use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run_json_lines(io::stdin().lock(), io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => report(&error),
    }
}

fn report(error: &io::Error) -> ExitCode {
    let _ = writeln!(io::stderr(), "avatar-routing: {error}");
    ExitCode::FAILURE
}
