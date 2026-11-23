use std::io::{self, Write};

pub trait Diagnostics {
    fn note(&mut self, line: &str);
}

pub struct StandardError;

impl Diagnostics for StandardError {
    fn note(&mut self, line: &str) {
        let _ = writeln!(io::stderr(), "{line}");
    }
}
