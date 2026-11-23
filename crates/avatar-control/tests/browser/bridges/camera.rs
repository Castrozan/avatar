use std::fs;
use std::path::Path;
use std::time::Duration;

use avatar_control::camera::{VideoDeviceSettings, VirtualCamera, VirtualCameraSettings};
use futures_util::SinkExt;
use serde_json::{Value, json};
use tokio::time::{sleep, timeout};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::ORIGIN;

use crate::cdp::serve_cdp;
use crate::support::{TestResult, message_id, result_reply, sent};
use crate::targets::serve_targets;

const FRAME: [u8; 16] = [7; 16];
const OUTPUT_URL: &str = "http://localhost:3000/?output=1";
const WAIT: Duration = Duration::from_secs(10);

fn modal_answer(message: &Value) -> Vec<String> {
    let value = json!({"result": {"type": "string", "value": "dismissed"}});
    vec![result_reply(message_id(message), value)]
}

fn video() -> VideoDeviceSettings {
    VideoDeviceSettings {
        frames_per_second: 30,
        width: 2,
        height: 2,
        video_device: "/dev/null".to_owned(),
    }
}

fn settings(devtools_port: u16, encoder: &Path) -> VirtualCameraSettings {
    VirtualCameraSettings {
        devtools_port,
        renderer_port: "3000".to_owned(),
        frame_port: 0,
        encoder_program: encoder.to_string_lossy().into_owned(),
        video: video(),
        open_settle: Duration::ZERO,
        modal_pause: Duration::from_millis(1),
    }
}

fn fake_encoder(directory: &Path) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;
    let path = directory.join("encoder");
    let script = format!(
        "#!/bin/sh\ncat > {}\n",
        directory.join("frames.bin").display()
    );
    fs::write(&path, script)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
    Ok(path)
}

async fn send_frame(address: std::net::SocketAddr, origin: &str) -> TestResult {
    let mut request = format!("ws://{address}/").into_client_request()?;
    request.headers_mut().insert(ORIGIN, origin.parse()?);
    let (mut socket, _) = connect_async(request).await?;
    socket.send(Message::Binary(FRAME.to_vec().into())).await?;
    Ok(())
}

async fn wait_for_encoded_bytes(path: &Path, length: usize) {
    while fs::metadata(path).map_or(0, |meta| meta.len()) < length as u64 {
        sleep(Duration::from_millis(10)).await;
    }
}

type Rig = (tempfile::TempDir, crate::cdp::CdpServer, VirtualCamera);

fn output_page(cdp_port: u16) -> Value {
    let debugger = format!("ws://127.0.0.1:{cdp_port}/devtools/page/1");
    json!({"id": "o", "type": "page", "url": OUTPUT_URL, "webSocketDebuggerUrl": debugger})
}

async fn start_rig() -> Result<Rig, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let cdp = serve_cdp(modal_answer).await?;
    let targets = serve_targets(vec![output_page(cdp.port)], None, "").await?;
    let encoder = fake_encoder(directory.path())?;
    let camera = VirtualCamera::start(&settings(targets.port, &encoder)).await?;
    Ok((directory, cdp, camera))
}

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
