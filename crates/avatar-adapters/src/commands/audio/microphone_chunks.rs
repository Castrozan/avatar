use std::io::{self, Read};
use std::process::{Child, Command, Stdio};

use super::playback::{capture_command, chunk_bytes};
use crate::commands::Programs;
use crate::process::terminate;

pub struct MicrophoneChunks {
    recorder: Child,
    chunk_bytes: usize,
}

impl MicrophoneChunks {
    pub fn start(programs: &Programs, rate: u32, device: &str) -> io::Result<Self> {
        let command = capture_command(programs, rate, device);
        let recorder = Command::new(&command[0])
            .args(&command[1..])
            .stdout(Stdio::piped())
            .spawn()?;
        Ok(Self {
            recorder,
            chunk_bytes: chunk_bytes(rate),
        })
    }
}

impl Iterator for MicrophoneChunks {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Vec<u8>> {
        let mut chunk = vec![0; self.chunk_bytes];
        self.recorder.stdout.as_mut()?.read_exact(&mut chunk).ok()?;
        Some(chunk)
    }
}

impl Drop for MicrophoneChunks {
    fn drop(&mut self) {
        terminate(&mut self.recorder);
    }
}
