use std::io;
use std::num::NonZeroUsize;
use std::process::Stdio;

use avatar_live::session::RATE;
use avatar_room_audio::resampler::LinearResampler;
use tokio::io::AsyncReadExt;
use tokio::process::{Child, ChildStdout, Command};

use crate::daemons::live::voice::VoiceHandle;
use crate::support::mac_capture::mac_capture_command;

const CHUNKS_PER_SECOND: u32 = 10;
const SAMPLE_WIDTH_BYTES: usize = 2;
const SEAT_MICROPHONE_RATE: u32 = 16000;

struct Capture {
    command: Vec<String>,
    rate: u32,
    resampler: Option<LinearResampler>,
}

fn parec_command(device: &str) -> Vec<String> {
    let fixed = "--raw --channels=1 --format=s16le --latency-msec=40";
    let client = "--client-name=avatar-live-microphone";
    let line = format!("parec --device={device} --rate={RATE} {fixed} {client}");
    line.split_whitespace().map(str::to_owned).collect()
}

fn rate_of(rate: u32) -> io::Result<NonZeroUsize> {
    let wide = usize::try_from(rate).ok().and_then(NonZeroUsize::new);
    wide.ok_or_else(|| io::Error::other("sample rate must be positive"))
}

fn seat_capture(device: &str) -> io::Result<Capture> {
    let resampler = LinearResampler::new(rate_of(SEAT_MICROPHONE_RATE)?, rate_of(RATE)?);
    Ok(Capture {
        command: mac_capture_command(device),
        rate: SEAT_MICROPHONE_RATE,
        resampler: Some(resampler),
    })
}

fn capture_of(device: &str) -> io::Result<Capture> {
    if cfg!(target_os = "macos") {
        return seat_capture(device);
    }
    let command = parec_command(device);
    Ok(Capture {
        command,
        rate: RATE,
        resampler: None,
    })
}

fn start_recorder(command: &[String]) -> io::Result<Child> {
    let (program, arguments) = command.split_first().ok_or(io::ErrorKind::InvalidInput)?;
    Command::new(program)
        .args(arguments)
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
}

fn chunk_bytes(rate: u32) -> usize {
    let samples = usize::try_from(rate / CHUNKS_PER_SECOND).unwrap_or_default();
    samples * SAMPLE_WIDTH_BYTES
}

fn converted(resampler: &mut Option<LinearResampler>, chunk: &[u8]) -> io::Result<Vec<u8>> {
    match resampler {
        Some(resampler) => resampler.resample_pcm(chunk).map_err(io::Error::other),
        None => Ok(chunk.to_vec()),
    }
}

async fn forward(
    mut output: ChildStdout,
    mut capture: Capture,
    voice: &VoiceHandle,
) -> io::Result<()> {
    let mut chunk = vec![0; chunk_bytes(capture.rate)];
    loop {
        output.read_exact(&mut chunk).await?;
        voice.send_audio(converted(&mut capture.resampler, &chunk)?);
    }
}

pub(crate) async fn stream_microphone(device: String, voice: VoiceHandle) -> io::Result<()> {
    let capture = capture_of(&device)?;
    let mut recorder = start_recorder(&capture.command)?;
    let output = recorder.stdout.take().ok_or(io::ErrorKind::BrokenPipe)?;
    forward(output, capture, &voice).await
}
