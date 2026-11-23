use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use avatar_floor::protocol::respond;

fn main() -> ExitCode {
    match serve() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn serve() -> io::Result<()> {
    let mut output = io::stdout().lock();
    for line in io::stdin().lock().lines() {
        reply(&mut output, &line?)?;
    }
    Ok(())
}

fn reply(output: &mut impl Write, line: &str) -> io::Result<()> {
    if line.trim().is_empty() {
        return Ok(());
    }
    writeln!(output, "{}", respond(line))?;
    output.flush()
}
