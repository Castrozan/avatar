use std::io::Write;

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::time::{Instant, timeout_at};
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::tungstenite::{Error, Message};

use crate::command::connect::Socket;
use crate::command::reaction::{Reaction, react};

pub(super) enum Step {
    Continue,
    Done(i32),
}

pub(super) struct Exchange<'run, Out: Write, Err: Write> {
    pub(super) socket: Socket,
    pub(super) command: Value,
    pub(super) stdout: &'run mut Out,
    pub(super) stderr: &'run mut Err,
}

impl<Out: Write, Err: Write> Exchange<'_, Out, Err> {
    pub(super) async fn run(mut self, deadline: Instant) -> i32 {
        let identify = json!({"type": "identify", "role": "agent"});
        let first = self.send(&identify).await;
        drive(&mut self, first, deadline).await
    }

    async fn send(&mut self, message: &Value) -> Step {
        match self.socket.send(Message::text(message.to_string())).await {
            Ok(()) => Step::Continue,
            Err(error) => self.fail(&error.to_string()),
        }
    }

    async fn next(&mut self, deadline: Instant) -> Step {
        match timeout_at(deadline, self.socket.next()).await {
            Err(_) => self.fail("Avatar command timed out"),
            Ok(None) => Step::Done(0),
            Ok(Some(Err(error))) if hung_up(&error) => Step::Done(0),
            Ok(Some(Err(error))) => self.fail(&error.to_string()),
            Ok(Some(Ok(message))) => self.handle(message).await,
        }
    }

    fn fail(&mut self, text: &str) -> Step {
        drop(writeln!(self.stderr, "{text}"));
        Step::Done(1)
    }

    async fn handle(&mut self, message: Message) -> Step {
        match message {
            Message::Text(text) => self.respond(text.as_str()).await,
            Message::Close(_) => Step::Done(0),
            _ => Step::Continue,
        }
    }

    async fn respond(&mut self, text: &str) -> Step {
        match serde_json::from_str::<Value>(text) {
            Ok(response) => self.apply(react(&response, &self.command)).await,
            Err(error) => self.fail(&error.to_string()),
        }
    }

    async fn finish(&mut self, printed: &str, failed: bool) -> Step {
        drop(writeln!(self.stdout, "{printed}"));
        drop(self.socket.close(None).await);
        Step::Done(i32::from(failed))
    }

    async fn apply(&mut self, reaction: Reaction) -> Step {
        match reaction {
            Reaction::SendCommand => self.send(&self.command.clone()).await,
            Reaction::Ignore => Step::Continue,
            Reaction::Finish { printed, failed } => self.finish(&printed, failed).await,
        }
    }
}

async fn drive<Out: Write, Err: Write>(
    exchange: &mut Exchange<'_, Out, Err>,
    first: Step,
    deadline: Instant,
) -> i32 {
    let mut step = first;
    while matches!(step, Step::Continue) {
        step = exchange.next(deadline).await;
    }
    match step {
        Step::Done(code) => code,
        Step::Continue => 0,
    }
}

fn hung_up(error: &Error) -> bool {
    matches!(
        error,
        Error::ConnectionClosed
            | Error::AlreadyClosed
            | Error::Protocol(ProtocolError::ResetWithoutClosingHandshake)
    )
}
