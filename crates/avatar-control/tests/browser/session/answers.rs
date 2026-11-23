use avatar_control::browser::DevToolsSession;
use serde_json::{Value, json};

use super::super::support::{TestResult, message_id, result_reply};

use super::*;

fn answer_nothing(_message: &Value) -> Vec<String> {
    Vec::new()
}

#[tokio::test]
async fn silent_chrome_times_out_with_did_not_answer() -> TestResult {
    let (session, recorded) = session_with(answer_nothing).await?;
    let outcome = session
        .send_and_await_result("Page.enable", json!({}), SHORT)
        .await;
    assert_eq!(failure_text(outcome).as_deref(), Some(NO_ANSWER));
    assert_eq!(sent_frames(&recorded)[0]["method"], "Page.enable");
    Ok(())
}

fn answer_without_result(message: &Value) -> Vec<String> {
    vec![json!({ "id": message_id(message) }).to_string()]
}

#[tokio::test]
async fn answer_without_result_resolves_to_null() -> TestResult {
    let (session, _recorded) = session_with(answer_without_result).await?;
    assert_eq!(call(&session, "Page.enable", json!({})).await?, Value::Null);
    Ok(())
}

fn answer_with_events(message: &Value) -> Vec<String> {
    vec![
        json!({"method": "Page.loadEventFired", "params": {"timestamp": 1}}).to_string(),
        "not json".to_owned(),
        json!({"id": 5, "result": {}}).to_string(),
        result_reply(message_id(message), json!({})),
    ]
}

#[tokio::test]
async fn events_and_answers_are_broadcast_and_invalid_frames_skipped() -> TestResult {
    let (session, _recorded) = session_with(answer_with_events).await?;
    let mut events = session.subscribe_events();
    call(&session, "Page.enable", json!({})).await?;
    let received = [
        events.recv().await?,
        events.recv().await?,
        events.recv().await?,
    ];
    assert_eq!(received, expected_events());
    Ok(())
}

fn expected_events() -> [Value; 3] {
    let loaded = json!({"method": "Page.loadEventFired", "params": {"timestamp": 1}});
    [
        loaded,
        json!({"id": 5, "result": {}}),
        json!({"id": 1, "result": {}}),
    ]
}

fn answer_numbered(message: &Value) -> Vec<String> {
    let number = if message["method"] == "Second.call" {
        2
    } else {
        1
    };
    vec![result_reply(message_id(message), json!({"n": number}))]
}

#[tokio::test]
async fn message_ids_continue_across_requests() -> TestResult {
    let (session, recorded) = session_with(answer_numbered).await?;
    let first = call(&session, "First.call", json!({})).await?;
    let second = call(&session, "Second.call", json!({})).await?;
    assert_eq!((first, second), (json!({"n": 1}), json!({"n": 2})));
    let ids: Vec<Value> = sent_frames(&recorded)
        .iter()
        .map(|frame| frame["id"].clone())
        .collect();
    assert_eq!(ids, vec![json!(1), json!(2)]);
    Ok(())
}
