use std::path::Path;
use std::process::{ExitStatus, Stdio};

use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};

use crate::synthesis::contract::SynthesisError;

pub(super) fn spawn_piper(
    program: &Path,
    model_path: &Path,
    audio_path: &Path,
) -> Result<Child, SynthesisError> {
    Ok(Command::new(program)
        .arg("--model")
        .arg(model_path)
        .arg("--output_file")
        .arg(audio_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?)
}

pub(super) async fn feed_text(child: &mut Child, text: &str) -> std::io::Result<()> {
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes()).await?;
    }
    Ok(())
}

pub(super) fn piper_outcome(status: ExitStatus) -> Result<(), SynthesisError> {
    if status.success() {
        return Ok(());
    }
    Err(SynthesisError::Failed(format!("piper failed: {status}")))
}
