use std::io::{self, Write};

use crate::session::Console;

#[derive(Debug, Clone, Copy, Default)]
pub struct StandardConsole;

impl Console for StandardConsole {
    fn print_line(&mut self, line: &str) {
        let _ = writeln!(io::stdout().lock(), "{line}");
    }

    fn print_error(&mut self, line: &str) {
        let _ = writeln!(io::stderr().lock(), "{line}");
    }
}
