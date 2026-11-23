use std::io;
use std::process::Stdio;

use avatar_live::session::RATE;
use tokio::io::AsyncReadExt;
use tokio::process::{Child, ChildStdout, Command};

use crate::daemons::live::voice::VoiceHandle;

const CHUNKS_PER_SECOND: u32 = 10;
const SAMPLE_WIDTH_BYTES: usize = 2;

fn start_recorder(device: &str) -> io::Result<Child> {
    Command::new("parec")
        .arg(format!("--device={device}"))
        .args(["--raw", &format!("--rate={RATE}"), "--channels=1"])
        .args(["--format=s16le", "--latency-msec=40"])
        .arg("--client-name=avatar-live-microphone")
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
}

fn chunk_bytes() -> usize {
    let samples = usize::try_from(RATE / CHUNKS_PER_SECOND).unwrap_or_default();
    samples * SAMPLE_WIDTH_BYTES
}

async fn forward(mut output: ChildStdout, voice: &VoiceHandle) -> io::Result<()> {
    let mut chunk = vec![0; chunk_bytes()];
    loop {
        output.read_exact(&mut chunk).await?;
        voice.send_audio(chunk.clone());
    }
}

pub(crate) async fn stream_microphone(device: String, voice: VoiceHandle) -> io::Result<()> {
    let mut recorder = start_recorder(&device)?;
    let output = recorder.stdout.take().ok_or(io::ErrorKind::BrokenPipe)?;
    forward(output, &voice).await
}
