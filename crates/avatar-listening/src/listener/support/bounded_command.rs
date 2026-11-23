use std::io::{self, Read};
use std::path::Path;
use std::process::{Child, ChildStdout, Command, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(10);

pub struct CommandOutput {
    pub code: i32,
    pub stdout: String,
}

fn timed_out(child: &mut Child) -> io::Result<Option<i32>> {
    child.kill()?;
    child.wait()?;
    Err(io::ErrorKind::TimedOut.into())
}

fn poll(child: &mut Child, deadline: Instant) -> io::Result<Option<i32>> {
    if let Some(status) = child.try_wait()? {
        return Ok(Some(status.code().unwrap_or(-1)));
    }
    if Instant::now() >= deadline {
        return timed_out(child);
    }
    Ok(None)
}

fn wait_until(child: &mut Child, deadline: Instant) -> io::Result<i32> {
    loop {
        match poll(child, deadline)? {
            Some(code) => return Ok(code),
            None => thread::sleep(POLL_INTERVAL),
        }
    }
}

type OutputReader = JoinHandle<io::Result<String>>;

fn read_in_background(mut pipe: ChildStdout) -> OutputReader {
    thread::spawn(move || {
        let mut text = String::new();
        pipe.read_to_string(&mut text).map(|_| text)
    })
}

fn spawn_reading(program: &Path, arguments: &[&str]) -> io::Result<(Child, OutputReader)> {
    let mut child = Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let pipe = child.stdout.take().ok_or(io::ErrorKind::BrokenPipe)?;
    Ok((child, read_in_background(pipe)))
}

fn collect(reader: OutputReader) -> io::Result<String> {
    reader
        .join()
        .map_err(|_| io::Error::other("output reader failed"))?
}

pub fn run_bounded(
    program: &Path,
    arguments: &[&str],
    timeout: Duration,
) -> io::Result<CommandOutput> {
    let deadline = Instant::now() + timeout;
    let (mut child, reader) = spawn_reading(program, arguments)?;
    let code = wait_until(&mut child, deadline)?;
    let stdout = collect(reader)?;
    Ok(CommandOutput { code, stdout })
}
