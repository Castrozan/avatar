use std::io::Write;

use avatar_conversation::talk::ports::Console;

pub(crate) struct Terminal;

impl Console for Terminal {
    fn print_out(&mut self, text: &str) {
        let mut stdout = std::io::stdout().lock();
        let _ = stdout
            .write_all(text.as_bytes())
            .and_then(|()| stdout.flush());
    }

    fn print_err(&mut self, text: &str) {
        let mut stderr = std::io::stderr().lock();
        let _ = stderr
            .write_all(text.as_bytes())
            .and_then(|()| stderr.flush());
    }
}
