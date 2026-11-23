use std::time::Duration;

use serde_json::json;

use super::{BrowserError, DevToolsSession, TargetLocator, WebSocketDevToolsSession};

#[derive(Debug, Clone)]
pub struct BridgeInjection {
    pub devtools_port: u16,
    pub renderer_port: String,
    pub settle: Duration,
}

pub async fn inject_bridge(
    injection: &BridgeInjection,
    bridge_script: &str,
) -> Result<(), BrowserError> {
    let debugger_url = renderer_debugger_url(injection).await?;
    let session = WebSocketDevToolsSession::connect(&debugger_url).await?;
    evaluate_then_close(&session, bridge_script, injection.settle).await;
    Ok(())
}

async fn evaluate_then_close(session: &impl DevToolsSession, script: &str, settle: Duration) {
    let params = json!({ "expression": script });
    let outcome = session
        .send_and_await_result("Runtime.evaluate", params, settle)
        .await;
    session.close();
    drop(outcome);
}

async fn renderer_debugger_url(injection: &BridgeInjection) -> Result<String, BrowserError> {
    let fragment = format!("localhost:{}", injection.renderer_port);
    let target = TargetLocator::new(injection.devtools_port)
        .find_page_target(&fragment)
        .await?;
    target
        .and_then(|found| found.web_socket_debugger_url)
        .ok_or_else(|| missing_renderer(injection.devtools_port))
}

fn missing_renderer(devtools_port: u16) -> BrowserError {
    BrowserError::Protocol {
        message: format!("Renderer tab not found on CDP port {devtools_port}"),
    }
}
