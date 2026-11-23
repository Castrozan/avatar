use std::io::Write;

use avatar_holder::floor_holder::Reporter;

pub struct LineReporter<Sink: Write> {
    sink: Sink,
}

impl<Sink: Write> LineReporter<Sink> {
    #[must_use]
    pub const fn new(sink: Sink) -> Self {
        Self { sink }
    }
}

impl<Sink: Write> Reporter for LineReporter<Sink> {
    fn report(&mut self, message: &str) {
        drop(writeln!(self.sink, "{message}"));
        drop(self.sink.flush());
    }
}
