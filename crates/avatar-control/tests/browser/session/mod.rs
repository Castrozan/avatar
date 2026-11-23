mod answers;

use std::error::Error;
use std::time::Duration;

use avatar_control::browser::{BrowserError, DevToolsSession, WebSocketDevToolsSession};
use serde_json::{Value, json};

use super::cdp::{Script, serve_cdp};
use super::support::{Recorded, TestResult, error_reply, message_id, result_reply, sent};

const GENEROUS: Duration = Duration::from_secs(5);

const SHORT: Duration = Duration::from_millis(50);

const NO_ANSWER: &str = "Chrome did not answer Page.enable";

async fn session_with(
    script: Script,
) -> Result<(WebSocketDevToolsSession, Recorded), Box<dyn Error>> {
    let server = serve_cdp(script).await?;
    let url = format!("ws://127.0.0.1:{}/devtools/page/1", server.port);
    Ok((
        WebSocketDevToolsSession::connect(&url).await?,
        server.received,
    ))
}

async fn call(
    session: &WebSocketDevToolsSession,
    method: &str,
    params: Value,
) -> Result<Value, BrowserError> {
    session
        .send_and_await_result(method, params, GENEROUS)
        .await
}

fn failure_text(outcome: Result<Value, BrowserError>) -> Option<String> {
    outcome.err().map(|error| error.to_string())
}

fn sent_frames(recorded: &Recorded) -> Vec<Value> {
    sent(recorded)
}

fn answer_evaluate(message: &Value) -> Vec<String> {
    vec![result_reply(
        message_id(message),
        json!({"result": {"value": 2}}),
    )]
}

#[tokio::test]
async fn single_request_resolves_with_result_and_sends_id_method_and_params() -> TestResult {
    let (session, recorded) = session_with(answer_evaluate).await?;
    let params = json!({"expression": "1+1"});
    let result = call(&session, "Runtime.evaluate", params.clone()).await?;
    assert_eq!(result, json!({"result": {"value": 2}}));
    let expected = json!({"id": 1, "method": "Runtime.evaluate", "params": params});
    assert_eq!(sent_frames(&recorded), vec![expected]);
    Ok(())
}

fn answer_second_before_first(message: &Value) -> Vec<String> {
    match message["method"].as_str() {
        Some("B.two") => vec![
            result_reply(2, json!({"which": "second"})),
            result_reply(1, json!({"which": "first"})),
        ],
        _ => Vec::new(),
    }
}

#[tokio::test]
async fn out_of_order_answers_resolve_their_own_requests() -> TestResult {
    let (session, recorded) = session_with(answer_second_before_first).await?;
    let first = call(&session, "A.one", json!({"k": 1}));
    let second = call(&session, "B.two", json!({"k": 2}));
    let (first, second) = tokio::join!(first, second);
    assert_eq!(first?, json!({"which": "first"}));
    assert_eq!(second?, json!({"which": "second"}));
    assert_eq!(sent_frames(&recorded).len(), 2);
    Ok(())
}

fn answer_stray_then_real(message: &Value) -> Vec<String> {
    vec![
        result_reply(99, json!({"ignored": true})),
        result_reply(message_id(message), json!({"ok": true})),
    ]
}

#[tokio::test]
async fn answer_with_an_unrelated_id_is_ignored() -> TestResult {
    let (session, _recorded) = session_with(answer_stray_then_real).await?;
    let result = call(&session, "Page.navigate", json!({"url": "x"})).await?;
    assert_eq!(result, json!({"ok": true}));
    Ok(())
}

fn answer_with_protocol_error(message: &Value) -> Vec<String> {
    vec![error_reply(message_id(message), "No target with given id")]
}

#[tokio::test]
async fn protocol_error_rejects_with_its_message() -> TestResult {
    let (session, _recorded) = session_with(answer_with_protocol_error).await?;
    let outcome = call(&session, "Runtime.evaluate", json!({"expression": "x"})).await;
    assert_eq!(
        failure_text(outcome).as_deref(),
        Some("No target with given id")
    );
    Ok(())
}
