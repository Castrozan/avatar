mod lifecycle;

use std::error::Error;
use std::fs;

use avatar_control::camera::{VideoDeviceFramePipeline, VideoDeviceSettings};
use tokio::sync::mpsc::UnboundedReceiver;

use super::support::{Scratch, TestResult, recorded_arguments};

fn default_settings() -> VideoDeviceSettings {
    VideoDeviceSettings {
        frames_per_second: 60,
        width: 1280,
        height: 720,
        video_device: "/dev/video10".to_owned(),
    }
}

async fn feed(encoder: &str, frame: Vec<u8>) -> Result<bool, Box<dyn Error>> {
    let (mut pipeline, _diagnostics) =
        VideoDeviceFramePipeline::start(encoder, &default_settings())?;
    let accepted = pipeline.write_frame(frame);
    pipeline.end();
    pipeline.finished().await?;
    Ok(accepted)
}

fn record_argv(scratch: &Scratch) -> String {
    format!(
        "printf '%s\\0' \"$@\" > {}\ncat > /dev/null",
        scratch.file("argv.bin")
    )
}

#[tokio::test]
async fn encoder_is_started_with_the_js_argv() -> TestResult {
    let scratch = Scratch::new("argv")?;
    let encoder = scratch.fake_encoder(&record_argv(&scratch))?;
    feed(&encoder, Vec::new()).await?;
    let arguments = recorded_arguments(&scratch.file("argv.bin"))?;
    assert_eq!(arguments, default_settings().encoder_arguments());
    Ok(())
}

#[tokio::test]
async fn frame_bytes_reach_the_encoder_unchanged() -> TestResult {
    let scratch = Scratch::new("bytes")?;
    let encoder = scratch.fake_encoder(&format!("cat > {}", scratch.file("stdin.bin")))?;
    let frame: Vec<u8> = (0..=255).collect();
    assert!(feed(&encoder, frame.clone()).await?);
    assert_eq!(fs::read(scratch.file("stdin.bin"))?, frame);
    Ok(())
}

#[tokio::test]
async fn frame_is_skipped_while_the_encoder_is_still_draining() -> TestResult {
    let scratch = Scratch::new("drain")?;
    let encoder = scratch.fake_encoder("exec sleep 30")?;
    let (pipeline, _diagnostics) = VideoDeviceFramePipeline::start(&encoder, &default_settings())?;
    assert!(pipeline.write_frame(vec![7; 4 * 1024 * 1024]));
    assert!(!pipeline.write_frame(vec![7; 16]));
    Ok(())
}

const PROGRESS_SCRIPT: &str =
    "printf 'frame=  1 fps=0\\n  Device or resource busy  \\n\\n' >&2\ncat > /dev/null";

async fn diagnostics_of(encoder: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let (mut pipeline, mut diagnostics) =
        VideoDeviceFramePipeline::start(encoder, &default_settings())?;
    pipeline.end();
    pipeline.finished().await?;
    Ok(drain_lines(&mut diagnostics).await)
}

async fn drain_lines(diagnostics: &mut UnboundedReceiver<String>) -> Vec<String> {
    let mut lines = Vec::new();
    while let Some(line) = diagnostics.recv().await {
        lines.push(line);
    }
    lines
}

#[tokio::test]
async fn stderr_progress_lines_are_filtered_and_trimmed() -> TestResult {
    let scratch = Scratch::new("stderr")?;
    let encoder = scratch.fake_encoder(PROGRESS_SCRIPT)?;
    assert_eq!(
        diagnostics_of(&encoder).await?,
        vec!["Device or resource busy".to_owned()]
    );
    Ok(())
}
