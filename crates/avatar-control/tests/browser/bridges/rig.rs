use std::fs;
use std::path::Path;
use std::time::Duration;

use avatar_control::camera::{VideoDeviceSettings, VirtualCamera, VirtualCameraSettings};
use futures_util::SinkExt;
use serde_json::{Value, json};
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::ORIGIN;

use crate::cdp::serve_cdp;
use crate::support::{TestResult, message_id, result_reply};
use crate::targets::serve_targets;

pub(crate) const FRAME: [u8; 16] = [7; 16];

pub(crate) const OUTPUT_URL: &str = "http://localhost:3000/?output=1";

pub(crate) const WAIT: Duration = Duration::from_secs(10);

pub(crate) fn modal_answer(message: &Value) -> Vec<String> {
    let value = json!({"result": {"type": "string", "value": "dismissed"}});
    vec![result_reply(message_id(message), value)]
}

pub(crate) fn video() -> VideoDeviceSettings {
    VideoDeviceSettings {
        frames_per_second: 30,
        width: 2,
        height: 2,
        video_device: "/dev/null".to_owned(),
    }
}

pub(crate) fn settings(devtools_port: u16, encoder: &Path) -> VirtualCameraSettings {
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

pub(crate) fn fake_encoder(
    directory: &Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
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

pub(crate) async fn send_frame(address: std::net::SocketAddr, origin: &str) -> TestResult {
    let mut request = format!("ws://{address}/").into_client_request()?;
    request.headers_mut().insert(ORIGIN, origin.parse()?);
    let (mut socket, _) = connect_async(request).await?;
    socket.send(Message::Binary(FRAME.to_vec().into())).await?;
    Ok(())
}

pub(crate) async fn wait_for_encoded_bytes(path: &Path, length: usize) {
    while fs::metadata(path).map_or(0, |meta| meta.len()) < length as u64 {
        sleep(Duration::from_millis(10)).await;
    }
}

pub(crate) type Rig = (tempfile::TempDir, crate::cdp::CdpServer, VirtualCamera);

pub(crate) fn output_page(cdp_port: u16) -> Value {
    let debugger = format!("ws://127.0.0.1:{cdp_port}/devtools/page/1");
    json!({"id": "o", "type": "page", "url": OUTPUT_URL, "webSocketDebuggerUrl": debugger})
}

pub(crate) async fn start_rig() -> Result<Rig, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let cdp = serve_cdp(modal_answer).await?;
    let targets = serve_targets(vec![output_page(cdp.port)], None, "").await?;
    let encoder = fake_encoder(directory.path())?;
    let camera = VirtualCamera::start(&settings(targets.port, &encoder)).await?;
    Ok((directory, cdp, camera))
}
