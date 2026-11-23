use std::io::{self, Write};

use avatar_seat::bridge::BridgeLog;

pub(super) struct StderrLog;

impl BridgeLog for StderrLog {
    fn log(&self, message: &str) {
        let _ = writeln!(io::stderr(), "{message}");
    }
}
