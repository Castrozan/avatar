use std::io::Write;

use crate::failure::{Failure, FailureKind};
use crate::frame::Frame;

fn os_failure(error: &std::io::Error) -> Failure {
    Failure::new(FailureKind::Os, &error.to_string())
}

fn write_flushed(output: &mut impl Write, bytes: &[u8]) -> Result<(), Failure> {
    output
        .write_all(bytes)
        .map_err(|error| os_failure(&error))?;
    output.flush().map_err(|error| os_failure(&error))
}

fn audio_of(frame: Frame) -> Option<Vec<u8>> {
    match frame {
        Frame::Binary(bytes) => Some(bytes),
        Frame::Text(_) => None,
    }
}

pub fn copy_call_audio(
    frames: impl Iterator<Item = Frame>,
    output: &mut impl Write,
) -> Result<(), Failure> {
    frames
        .filter_map(audio_of)
        .try_for_each(|bytes| write_flushed(output, &bytes))
}

#[must_use]
pub fn call_capture_exit(outcome: &Result<(), Failure>) -> i32 {
    i32::from(outcome.is_err())
}
