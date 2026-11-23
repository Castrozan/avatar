mod exchange;

use std::io::Write;
use std::time::Duration;

use serde_json::Value;
use tokio::time::Instant;

use super::connect::{Socket, connect_to_starting_server};
use super::parse::parse_command;
use super::seat::seat_is_attached;
use super::speech_output::{needs_seat_lookup, with_speech_output};
use exchange::Exchange;

#[derive(Debug, Clone)]
pub struct CommandEndpoints {
    pub control_url: String,
    pub seat_health_url: String,
    pub seat_patience: Duration,
    pub start_patience: Duration,
    pub start_retry: Duration,
    pub speech_timeout: Duration,
    pub command_timeout: Duration,
}

impl Default for CommandEndpoints {
    fn default() -> Self {
        Self {
            control_url: "ws://127.0.0.1:8765".to_owned(),
            seat_health_url: "http://127.0.0.1:8769/health".to_owned(),
            seat_patience: Duration::from_millis(300),
            start_patience: Duration::from_secs(30),
            start_retry: Duration::from_millis(250),
            speech_timeout: Duration::from_mins(5),
            command_timeout: Duration::from_secs(10),
        }
    }
}

pub async fn run_command(
    arguments: &[String],
    endpoints: &CommandEndpoints,
    command_id: &str,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let command = match parse_command(arguments) {
        Ok(command) => command,
        Err(usage) => return report(stderr, usage),
    };
    let command = resolve_output(command, endpoints).await;
    exchange_command(command, command_id, endpoints, stdout, stderr).await
}

fn report(stderr: &mut impl Write, text: &str) -> i32 {
    drop(writeln!(stderr, "{text}"));
    1
}

async fn resolve_output(command: Value, endpoints: &CommandEndpoints) -> Value {
    if !needs_seat_lookup(&command) {
        return command;
    }
    let attached = seat_is_attached(&endpoints.seat_health_url, endpoints.seat_patience).await;
    with_speech_output(command, attached)
}

async fn exchange_command(
    mut command: Value,
    command_id: &str,
    endpoints: &CommandEndpoints,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    command["id"] = Value::from(command_id);
    let (patience, retry) = (endpoints.start_patience, endpoints.start_retry);
    match connect_to_starting_server(&endpoints.control_url, patience, retry).await {
        Ok(socket) => converse(socket, command, endpoints, (stdout, stderr)).await,
        Err(error) => report(stderr, &error.to_string()),
    }
}

async fn converse<Out: Write, Err: Write>(
    socket: Socket,
    command: Value,
    endpoints: &CommandEndpoints,
    (stdout, stderr): (&mut Out, &mut Err),
) -> i32 {
    let deadline = deadline_for(&command, endpoints);
    let exchange = Exchange {
        socket,
        command,
        stdout,
        stderr,
    };
    exchange.run(deadline).await
}

fn deadline_for(command: &Value, endpoints: &CommandEndpoints) -> Instant {
    let speaking = command.get("type") == Some(&Value::from("speak"));
    let allowance = if speaking {
        endpoints.speech_timeout
    } else {
        endpoints.command_timeout
    };
    Instant::now() + allowance
}
