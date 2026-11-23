use std::io::{self, Write};

use avatar_live::boundaries::Console;

#[derive(Clone, Copy)]
pub(crate) struct StandardOutput;

impl Console for StandardOutput {
    fn print(&mut self, line: &str) {
        let mut output = io::stdout().lock();
        let _ = writeln!(output, "{line}").and_then(|()| output.flush());
    }
}
