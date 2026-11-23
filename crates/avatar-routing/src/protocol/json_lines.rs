use super::dispatch::respond;
use crate::names::vocabulary::{Vocabulary, frequency_ranks};
use std::io::{self, BufRead, Write};

pub fn run_json_lines(input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    let vocabulary = frequency_ranks();
    input
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
