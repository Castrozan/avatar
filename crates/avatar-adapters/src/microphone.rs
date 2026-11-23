use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use avatar_wake::listener::Microphone;

use crate::process::terminate;

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
    arguments: Vec<String>,
    capture: Option<Child>,
}

impl ParecMicrophone {
    #[must_use]
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self::relaying(program, capture_arguments())
    }

    #[must_use]
    pub fn relaying(program: impl Into<PathBuf>, arguments: Vec<String>) -> Self {
        Self {
            program: program.into(),
            arguments,
            capture: None,
        }
    }
}

impl Microphone for ParecMicrophone {
    fn start(&mut self) {
        self.capture = Command::new(&self.program)
            .args(&self.arguments)
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
