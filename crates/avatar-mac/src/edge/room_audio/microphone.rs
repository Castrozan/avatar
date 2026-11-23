use std::io::{self, Read};
use std::num::NonZeroUsize;
use std::process::{Child, Command, Stdio};

use avatar_room_audio::resampler::LinearResampler;

use super::{MICROPHONE_CHUNK_MILLISECONDS, SEAT_MICROPHONE_RATE, chunk_bytes};

pub struct RoomMicrophone {
    capture: Child,
    resampler: LinearResampler,
    chunk_bytes: usize,
}

fn rate_of(rate: u32) -> io::Result<NonZeroUsize> {
    let wide = usize::try_from(rate).ok();
    wide.and_then(NonZeroUsize::new)
        .ok_or_else(|| io::Error::other("sample rate must be positive"))
}

fn spawn_capture(command: &[String]) -> io::Result<Child> {
    let (program, arguments) = command
        .split_first()
        .ok_or_else(|| io::Error::other("empty capture command"))?;
    let mut capture = Command::new(program);
    capture.args(arguments).stdout(Stdio::piped()).spawn()
}

impl RoomMicrophone {
    pub fn start(command: &[String], rate: u32) -> io::Result<Self> {
        let capture = spawn_capture(command)?;
        let resampler = LinearResampler::new(rate_of(SEAT_MICROPHONE_RATE)?, rate_of(rate)?);
        let chunk_bytes = chunk_bytes(SEAT_MICROPHONE_RATE, MICROPHONE_CHUNK_MILLISECONDS);
        Ok(Self {
            capture,
            resampler,
            chunk_bytes,
        })
    }
}

impl Iterator for RoomMicrophone {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Vec<u8>> {
        let mut chunk = vec![0; self.chunk_bytes];
        self.capture.stdout.as_mut()?.read_exact(&mut chunk).ok()?;
        self.resampler.resample_pcm(&chunk).ok()
    }
}

impl Drop for RoomMicrophone {
    fn drop(&mut self) {
        let _ = self.capture.kill();
        let _ = self.capture.wait();
    }
}
