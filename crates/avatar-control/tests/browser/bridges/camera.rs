use std::fs;

use avatar_control::camera::VirtualCamera;
use tokio::time::timeout;

use crate::bridges::rig::{
    FRAME, WAIT, fake_encoder, send_frame, settings, start_rig, wait_for_encoded_bytes,
};
use crate::support::{TestResult, sent};
use crate::targets::serve_targets;

#[tokio::test]
async fn frames_from_the_renderer_reach_the_encoder_after_the_modal_is_dismissed() -> TestResult {
    let (directory, cdp, camera) = start_rig().await?;
    let encoded = directory.path().join("frames.bin");
    send_frame(camera.frame_address(), "http://localhost:3000").await?;
    timeout(WAIT, wait_for_encoded_bytes(&encoded, FRAME.len())).await?;
    assert_eq!(camera.stop().await, 1);
    assert_eq!(fs::read(encoded)?, FRAME.to_vec());
    assert_eq!(sent(&cdp.received)[0]["method"], "Runtime.evaluate");
    Ok(())
}

#[tokio::test]
async fn a_missing_output_tab_stops_the_start_with_the_node_message() -> TestResult {
    let directory = tempfile::tempdir()?;
    let targets = serve_targets(Vec::new(), None, "").await?;
    let encoder = fake_encoder(directory.path())?;
    let outcome = VirtualCamera::start(&settings(targets.port, &encoder)).await;
    let message = outcome.err().map(|error| error.to_string());
    let expected = "ChatVRM tab not found after auto-open. Is the renderer running on port 3000?";
    assert_eq!(message.as_deref(), Some(expected));
    Ok(())
}

#[tokio::test]
async fn frames_from_a_foreign_origin_are_refused() -> TestResult {
    let (_directory, _cdp, camera) = start_rig().await?;
    let refused = send_frame(camera.frame_address(), "http://evil.example").await;
    assert!(refused.is_err());
    assert_eq!(camera.stop().await, 0);
    Ok(())
}
