use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use avatar_wake::listener::Microphone;
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

const FRAME_BYTES: u64 = 1024;

const CAPTURE_ARGUMENTS: [&str; 7] = [
    "--device=@DEFAULT_SOURCE@",
    "--client-name=avatar-wake",
    "--raw",
    "--format=s16le",
    "--rate=16000",
    "--channels=1",
    "--latency-msec=100",
];

#[must_use]
pub fn capture_arguments() -> Vec<String> {
    CAPTURE_ARGUMENTS.map(str::to_owned).to_vec()
}

pub struct ParecMicrophone {
    program: PathBuf,
    capture: Option<Child>,
}

impl ParecMicrophone {
    #[must_use]
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            capture: None,
        }
    }
}

impl Microphone for ParecMicrophone {
    fn start(&mut self) {
        self.capture = Command::new(&self.program)
            .args(capture_arguments())
            .stdout(Stdio::piped())
            .spawn()
            .ok();
    }

    fn next_frame(&mut self) -> Option<Vec<u8>> {
        let stdout = self.capture.as_mut()?.stdout.as_mut()?;
        let mut frame = Vec::new();
        stdout.take(FRAME_BYTES).read_to_end(&mut frame).ok()?;
        (!frame.is_empty()).then_some(frame)
    }

    fn stop(&mut self) {
        let _ = self
            .capture
            .take()
            .map(|mut capture| terminate(&mut capture));
    }
}

fn terminate(capture: &mut Child) {
    let _ = i32::try_from(capture.id()).map(|pid| kill(Pid::from_raw(pid), Signal::SIGTERM));
    let _ = capture.wait();
}
