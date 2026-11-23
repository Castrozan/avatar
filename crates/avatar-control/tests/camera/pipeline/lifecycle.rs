use std::error::Error;

use avatar_control::camera::VideoDeviceFramePipeline;

use super::super::support::{Scratch, TestResult};

use super::*;

#[tokio::test]
async fn encoder_exit_code_is_reported() -> TestResult {
    let scratch = Scratch::new("exit")?;
    let encoder = scratch.fake_encoder("exit 3")?;
    let (mut pipeline, _diagnostics) =
        VideoDeviceFramePipeline::start(&encoder, &default_settings())?;
    assert_eq!(pipeline.finished().await?.code(), Some(3));
    Ok(())
}

#[tokio::test]
async fn frame_is_refused_after_the_pipeline_is_ended() -> TestResult {
    let scratch = Scratch::new("ended")?;
    let encoder = scratch.fake_encoder("cat > /dev/null")?;
    assert!(!feed_after_end(&encoder)?);
    Ok(())
}

fn feed_after_end(encoder: &str) -> Result<bool, Box<dyn Error>> {
    let (mut pipeline, _diagnostics) =
        VideoDeviceFramePipeline::start(encoder, &default_settings())?;
    pipeline.end();
    Ok(pipeline.write_frame(vec![1; 4]))
}

#[tokio::test]
async fn missing_encoder_program_fails_to_start() -> TestResult {
    let scratch = Scratch::new("missing")?;
    let outcome =
        VideoDeviceFramePipeline::start(&scratch.file("no-such-encoder"), &default_settings());
    assert!(outcome.is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound));
    Ok(())
}
