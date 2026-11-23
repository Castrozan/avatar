use std::time::Duration;

use serde_json::json;
use tokio::time::sleep;

use crate::browser::{BrowserError, DevToolsSession};

const DEVTOOLS_TIMEOUT: Duration = Duration::from_secs(10);
const DISMISS_START_MODAL_EXPRESSION: &str = r#"(() => {
  const btn = Array.from(document.querySelectorAll("button")).find(b => b.textContent.trim() === "Start");
  if (btn) { btn.click(); return "dismissed"; }
  return "no modal";
})()"#;

pub(super) async fn dismiss_start_modal(
    session: &impl DevToolsSession,
    pause: Duration,
) -> Result<String, BrowserError> {
    let status = evaluate_modal(session).await?;
    if status == "dismissed" {
        sleep(pause).await;
    }
    Ok(status)
}

async fn evaluate_modal(session: &impl DevToolsSession) -> Result<String, BrowserError> {
    let params = json!({ "expression": DISMISS_START_MODAL_EXPRESSION });
    let result = session
        .send_and_await_result("Runtime.evaluate", params, DEVTOOLS_TIMEOUT)
        .await?;
    Ok(modal_status(&result))
}

fn modal_status(result: &serde_json::Value) -> String {
    result["result"]["value"]
        .as_str()
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_owned()
}
