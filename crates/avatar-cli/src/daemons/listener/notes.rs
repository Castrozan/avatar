use std::io::{self, Write};

use avatar_listening::listener::diagnostics::Diagnostics;

#[derive(Clone, Copy)]
pub(super) struct StandardErrorNotes;

impl Diagnostics for StandardErrorNotes {
    fn note(&mut self, line: &str) {
        let _ = writeln!(io::stderr(), "{line}");
    }
}
