use super::endpoint::SeatEndpoint;
use super::wav::wav_samples;
use crate::failure::Failure;

pub trait Speakers {
    fn play(&mut self, recording: &str) -> Result<(), Failure>;
}

pub fn play_recording<Endpoint: SeatEndpoint>(
    endpoint: &mut Endpoint,
    speakers: &mut impl Speakers,
    recording: &str,
    read_recording: impl FnOnce(&str) -> Result<Vec<u8>, Failure>,
) -> Result<(), Failure> {
    if endpoint.attached() {
        let samples = wav_samples(&read_recording(recording)?)?;
        return endpoint.play(&samples);
    }
    speakers.play(recording)
}

pub fn hush_and_leave(endpoint: &mut impl SeatEndpoint) -> i32 {
    endpoint.hush();
    0
}
