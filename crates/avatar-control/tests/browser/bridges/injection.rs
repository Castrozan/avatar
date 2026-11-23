use std::time::Duration;

use avatar_control::browser::{BridgeInjection, inject_bridge};
use serde_json::{Value, json};

use crate::cdp::serve_cdp;
use crate::support::{TestResult, message_id, result_reply, sent};
use crate::targets::serve_targets;

const SCRIPT: &str = "window.__bridge = true;";

fn evaluated(message: &Value) -> Vec<String> {
    vec![result_reply(
        message_id(message),
        json!({"result": {"type": "undefined"}}),
    )]
}

fn silent(_message: &Value) -> Vec<String> {
    Vec::new()
}

fn injection(devtools_port: u16) -> BridgeInjection {
    BridgeInjection {
        devtools_port,
        renderer_port: "3000".to_owned(),
        settle: Duration::from_millis(100),
    }
}

fn renderer_page(cdp_port: u16, url: &str) -> Value {
    let debugger = format!("ws://127.0.0.1:{cdp_port}/devtools/page/1");
    json!({"id": "renderer", "type": "page", "url": url, "webSocketDebuggerUrl": debugger})
}

#[tokio::test]
async fn the_bridge_script_is_evaluated_in_the_renderer_tab() -> TestResult {
    let cdp = serve_cdp(evaluated).await?;
    let page = renderer_page(cdp.port, "http://localhost:3000/");
    let targets = serve_targets(vec![page], None, "").await?;
    inject_bridge(&injection(targets.port), SCRIPT).await?;
    let messages = sent(&cdp.received);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["method"], "Runtime.evaluate");
    assert_eq!(messages[0]["params"]["expression"], SCRIPT);
    Ok(())
}

#[tokio::test]
async fn a_missing_renderer_tab_is_reported_with_the_devtools_port() -> TestResult {
    let targets = serve_targets(Vec::new(), None, "").await?;
    let outcome = inject_bridge(&injection(targets.port), SCRIPT).await;
    let expected = format!("Renderer tab not found on CDP port {}", targets.port);
    assert_eq!(outcome.err().map(|error| error.to_string()), Some(expected));
    Ok(())
}

#[tokio::test]
async fn a_tab_on_another_host_name_is_not_the_renderer() -> TestResult {
    let cdp = serve_cdp(evaluated).await?;
    let page = renderer_page(cdp.port, "http://127.0.0.1:3000/");
    let targets = serve_targets(vec![page], None, "").await?;
    assert!(
        inject_bridge(&injection(targets.port), SCRIPT)
            .await
            .is_err()
    );
    assert!(sent(&cdp.received).is_empty());
    Ok(())
}

#[tokio::test]
async fn an_unanswered_evaluation_still_finishes_after_the_settle_time() -> TestResult {
    let cdp = serve_cdp(silent).await?;
    let page = renderer_page(cdp.port, "http://localhost:3000/");
    let targets = serve_targets(vec![page], None, "").await?;
    inject_bridge(&injection(targets.port), SCRIPT).await?;
    assert_eq!(sent(&cdp.received).len(), 1);
    Ok(())
}
