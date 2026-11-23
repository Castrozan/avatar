use std::io;
use std::process::Stdio;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

use crate::bridge::tools::{AudioTools, Capture, Playback};

pub struct SpawnedTools;

pub struct SpawnedPlayback {
    child: Child,
    input: ChildStdin,
}

pub struct SpawnedCapture {
    child: Child,
    output: ChildStdout,
}

fn spawn(command: &[String], input: bool) -> io::Result<Child> {
    let (program, arguments) = command
        .split_first()
        .ok_or_else(|| io::Error::other("empty command"))?;
    let mut prepared = Command::new(program);
    prepared.args(arguments).kill_on_drop(true);
    prepared.stdin(if input { Stdio::piped() } else { Stdio::null() });
    prepared.stdout(if input { Stdio::null() } else { Stdio::piped() });
    prepared.spawn()
}

async fn end_child(child: &mut Child) {
    let pid = child.id().and_then(|raw| i32::try_from(raw).ok());
    if let Some(raw) = pid {
        let _ = kill(Pid::from_raw(raw), Signal::SIGTERM);
    }
    let _ = child.wait().await;
}

impl AudioTools for SpawnedTools {
    type Playback = SpawnedPlayback;
    type Capture = SpawnedCapture;

    async fn playback(&self, command: Vec<String>) -> io::Result<SpawnedPlayback> {
        let mut child = spawn(&command, true)?;
        let input = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("no stdin"))?;
        Ok(SpawnedPlayback { child, input })
    }

    async fn capture(&self, command: Vec<String>) -> io::Result<SpawnedCapture> {
        let mut child = spawn(&command, false)?;
        let output = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("no stdout"))?;
        Ok(SpawnedCapture { child, output })
    }
}

impl Playback for SpawnedPlayback {
    async fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.input.write_all(bytes).await?;
        self.input.flush().await
    }

    async fn end(&mut self) {
        end_child(&mut self.child).await;
    }
}

impl Capture for SpawnedCapture {
    async fn read(&mut self, limit: usize) -> io::Result<Vec<u8>> {
        let mut chunk = vec![0; limit];
        let count = self.output.read(&mut chunk).await?;
        chunk.truncate(count);
        Ok(chunk)
    }

    async fn end(&mut self) {
        end_child(&mut self.child).await;
    }
}
