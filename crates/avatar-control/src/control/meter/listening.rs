use std::time::Duration;

use std::io;

use tokio::io::AsyncReadExt;
use tokio::process::{Child, ChildStdout};
use tokio::sync::watch;
use tokio::time::{Interval, interval};

use super::level::LevelAccumulator;
use super::sink::LevelSink;

const READ_BYTES: usize = 4096;

pub(super) enum Outcome {
    RecorderEnded,
    Stopped,
}

enum Event {
    Level(f64),
    Ended,
    Report,
    Stopped,
}

pub(super) struct Listening<Sink: LevelSink> {
    stdout: ChildStdout,
    accumulator: LevelAccumulator,
    level: f64,
    reports: Interval,
    sink: Sink,
}

impl<Sink: LevelSink> Listening<Sink> {
    pub(super) fn new(child: &mut Child, report_every: Duration, sink: &Sink) -> Option<Self> {
        Some(Self {
            stdout: child.stdout.take()?,
            accumulator: LevelAccumulator::default(),
            level: 0.0,
            reports: interval(report_every),
            sink: sink.clone(),
        })
    }

    pub(super) async fn run(mut self, stopped: &mut watch::Receiver<bool>) -> Outcome {
        drive(&mut self, stopped).await
    }

    async fn apply(&mut self, event: Event) -> Option<Outcome> {
        match event {
            Event::Ended => return Some(Outcome::RecorderEnded),
            Event::Stopped => return Some(Outcome::Stopped),
            Event::Level(level) => self.level = level,
            Event::Report => self.report().await,
        }
        None
    }

    async fn step(&mut self, stopped: &mut watch::Receiver<bool>) -> Option<Outcome> {
        let event = self.next_event(stopped).await;
        self.apply(event).await
    }

    async fn report(&mut self) {
        self.sink.send_level(self.level).await;
        self.level = 0.0;
    }

    async fn next_event(&mut self, stopped: &mut watch::Receiver<bool>) -> Event {
        tokio::select! {
            event = self.heard_or_report() => event,
            _ = stopped.wait_for(|requested| *requested) => Event::Stopped,
        }
    }

    async fn heard_or_report(&mut self) -> Event {
        let mut buffer = [0_u8; READ_BYTES];
        tokio::select! {
            read = self.stdout.read(&mut buffer) => heard(&mut self.accumulator, &read, &buffer),
            _ = self.reports.tick() => Event::Report,
        }
    }
}

fn heard(accumulator: &mut LevelAccumulator, read: &io::Result<usize>, buffer: &[u8]) -> Event {
    match read {
        Ok(count) if *count > 0 => {
            Event::Level(accumulator.push(buffer.get(..*count).unwrap_or(&[])))
        }
        _ => Event::Ended,
    }
}

async fn drive<Sink: LevelSink>(
    listening: &mut Listening<Sink>,
    stopped: &mut watch::Receiver<bool>,
) -> Outcome {
    let mut outcome = listening.step(stopped).await;
    while outcome.is_none() {
        outcome = listening.step(stopped).await;
    }
    outcome.unwrap_or(Outcome::Stopped)
}
