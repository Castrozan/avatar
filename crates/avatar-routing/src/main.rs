use avatar_routing::names::vocabulary::{Vocabulary, frequency_ranks};
use avatar_routing::protocol::dispatch::respond;
use std::io::{self, BufRead, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    match serve() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => report(&error),
    }
}

fn serve() -> io::Result<()> {
    let vocabulary = frequency_ranks();
    let mut output = io::stdout().lock();
    io::stdin()
        .lock()
        .lines()
        .try_for_each(|line| answer(&line?, vocabulary.as_ref(), &mut output))
}

fn answer(line: &str, vocabulary: Option<&Vocabulary>, output: &mut impl Write) -> io::Result<()> {
    if line.trim().is_empty() {
        return Ok(());
    }
    writeln!(output, "{}", respond(line, vocabulary))?;
    output.flush()
}

fn report(error: &io::Error) -> ExitCode {
    let _ = writeln!(io::stderr(), "avatar-routing: {error}");
    ExitCode::FAILURE
}
