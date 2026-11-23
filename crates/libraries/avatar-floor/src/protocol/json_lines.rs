use super::dispatch::respond;
use std::io::{self, BufRead, Write};

pub fn run_json_lines(input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    for line in input.lines() {
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
