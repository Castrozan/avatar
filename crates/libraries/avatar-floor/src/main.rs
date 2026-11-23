use avatar_floor::protocol::json_lines::run_json_lines;
use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run_json_lines(io::stdin().lock(), io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
