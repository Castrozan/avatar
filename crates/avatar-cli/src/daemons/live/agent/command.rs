use std::io::{self, Read};
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use avatar_live::agent::COMMAND_TIMEOUT_SECONDS;
use avatar_live::boundaries::CommandOutput;
use tokio::process::{Child, Command};
use tokio::task::spawn_blocking;
use tokio::time::timeout;

fn failed(output: String) -> CommandOutput {
    CommandOutput { code: None, output }
}

fn spawn(words: &[String], sink: io::PipeWriter) -> io::Result<Child> {
    let (program, arguments) = words.split_first().ok_or(io::ErrorKind::InvalidInput)?;
    Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(sink.try_clone()?)
        .stderr(sink)
        .kill_on_drop(true)
        .spawn()
}

fn read_all(mut reader: io::PipeReader) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn output_of(status: ExitStatus, bytes: &[u8]) -> CommandOutput {
    CommandOutput {
        code: status.code(),
        output: String::from_utf8_lossy(bytes).into_owned(),
    }
}

async fn capture(words: &[String]) -> io::Result<CommandOutput> {
    let (reader, writer) = io::pipe()?;
    let mut child = spawn(words, writer)?;
    let reading = spawn_blocking(move || read_all(reader));
    let status = child.wait().await?;
    let bytes = reading.await.map_err(io::Error::other)??;
    Ok(output_of(status, &bytes))
}

pub(super) async fn run_bounded(words: &[String]) -> CommandOutput {
    let limit = Duration::from_secs(u64::from(COMMAND_TIMEOUT_SECONDS));
    match timeout(limit, capture(words)).await {
        Ok(Ok(output)) => output,
        Ok(Err(failure)) => failed(failure.to_string()),
        Err(_) => failed(format!(
            "a2a gave no reply within {COMMAND_TIMEOUT_SECONDS} s"
        )),
    }
}
