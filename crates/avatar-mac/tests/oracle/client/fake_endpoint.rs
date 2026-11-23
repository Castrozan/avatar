use avatar_mac::client::{SeatEndpoint, StreamConnection};
use avatar_mac::failure::Failure;
use serde_json::{Value, json};

use crate::support::hex_text;
use crate::world::Log;

pub(crate) struct FakeConnection {
    log: Log,
    quiet: bool,
    swallow_header: bool,
}

pub(crate) struct FakeEndpoint {
    pub(crate) log: Log,
    pub(crate) attached: bool,
    pub(crate) wait_fails: bool,
    pub(crate) quiet_open: bool,
}

impl FakeEndpoint {
    pub(crate) fn new(log: &Log, attached: bool) -> Self {
        let log = log.clone();
        Self {
            log,
            attached,
            wait_fails: false,
            quiet_open: false,
        }
    }
}

fn sent_entry(header: bool, quiet: bool, data: &[u8]) -> Option<Value> {
    let text = String::from_utf8_lossy(data).into_owned();
    match (header, quiet) {
        (true, true) => None,
        (true, false) => Some(json!(["send", text])),
        _ => Some(json!(["send", hex_text(data)])),
    }
}

fn waited(quiet: bool, seconds: f64, message: &str) -> Option<Value> {
    (!quiet).then(|| json!(["wait", seconds, message]))
}

fn connected(quiet: bool) -> Value {
    if quiet {
        json!(["open", 24000])
    } else {
        json!(["connect"])
    }
}

impl StreamConnection for FakeConnection {
    fn send(&mut self, data: &[u8]) -> Result<(), Failure> {
        let header = std::mem::take(&mut self.swallow_header);
        self.log.extend(sent_entry(header, self.quiet, data));
        Ok(())
    }

    fn close(&mut self) {
        self.log.push(json!(["close"]));
    }
}

impl SeatEndpoint for FakeEndpoint {
    type Connection = FakeConnection;

    fn attached(&mut self) -> bool {
        self.attached
    }

    fn hush(&mut self) {
        self.log.push(json!(["hush"]));
    }

    fn play(&mut self, pcm: &[u8]) -> Result<(), Failure> {
        self.log.push(json!(["seat_play", hex_text(pcm)]));
        Ok(())
    }

    fn wait_until_attached(&mut self, seconds: f64, message: &str) -> Result<(), Failure> {
        self.log.extend(waited(self.quiet_open, seconds, message));
        self.wait_fails
            .then(|| Failure::timeout(message))
            .map_or(Ok(()), Err)
    }

    fn connect_stream(&mut self) -> Result<FakeConnection, Failure> {
        self.log.push(connected(self.quiet_open));
        let (log, quiet) = (self.log.clone(), self.quiet_open);
        Ok(FakeConnection {
            log,
            quiet,
            swallow_header: true,
        })
    }
}
