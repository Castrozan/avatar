use std::io::{self, Read};
use std::process::{Child, ChildStdout, Command, Stdio};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use super::source::{CaptureLauncher, CaptureStream};
use super::speech::FRAME_BYTES;

pub struct ProcessCapture<Command> {
    command_of: Command,
}

pub struct RunningCapture {
    child: Child,
    output: ChildStdout,
}

impl<CommandOf: Fn(&str) -> Vec<String>> ProcessCapture<CommandOf> {
    pub fn new(command_of: CommandOf) -> Self {
        Self { command_of }
    }
}

impl<CommandOf: Fn(&str) -> Vec<String>> CaptureLauncher for ProcessCapture<CommandOf> {
    type Stream = RunningCapture;

    fn launch(&mut self, device: &str) -> io::Result<RunningCapture> {
        let words = (self.command_of)(device);
        let (program, arguments) = words.split_first().ok_or(io::ErrorKind::InvalidInput)?;
        let mut child = Command::new(program)
            .args(arguments)
            .stdout(Stdio::piped())
            .spawn()?;
        let output = child.stdout.take().ok_or(io::ErrorKind::BrokenPipe)?;
        Ok(RunningCapture { child, output })
    }
}

fn fill(output: &mut impl Read, frame: &mut [u8]) -> usize {
    let mut filled = 0;
    while filled < frame.len() {
        match output.read(&mut frame[filled..]) {
            Ok(0) | Err(_) => return filled,
            Ok(count) => filled += count,
        }
    }
    filled
}

impl CaptureStream for RunningCapture {
    fn read_frame(&mut self) -> Option<Vec<u8>> {
        let mut frame = vec![0; FRAME_BYTES];
        let filled = fill(&mut self.output, &mut frame);
        frame.truncate(filled);
        (filled > 0).then_some(frame)
    }

    fn end(&mut self) {
        let terminate = |pid| kill(Pid::from_raw(pid), Signal::SIGTERM);
        let _ = i32::try_from(self.child.id()).map(terminate);
        let _ = self.child.wait();
    }
}
