use avatar_live::client::GptLiveClient;
use serde_json::{Value, json};

use super::connect_to;
use super::server::{MockServer, start};
use crate::support::{Outcome, block_on, cases, events_json, same};

const MESSAGES: &str = include_str!("fixtures/client_messages.json");

type Failure = Box<dyn std::error::Error>;

async fn send_recorded_audio(client: &mut GptLiveClient) -> Outcome {
    client.send_audio(&[0, 1, 2, 255]).await?;
    Ok(client.send_audio(&[]).await?)
}

async fn send_recorded_deliveries(client: &mut GptLiveClient) -> Outcome {
    client.deliver(Some("task-1"), "Checking.", false).await?;
    client.deliver(Some("task-1"), "Done.", true).await?;
    Ok(client.deliver(None, "\u{e7}\u{e3}o", true).await?)
}

async fn received_after_handshake(server: &mut MockServer, count: usize) -> Vec<Value> {
    let mut received = Vec::new();
    for _ in 0..count {
        received.push(server.received.recv().await.unwrap_or_default());
    }
    received.split_off(2)
}

fn started() -> Value {
    json!({"type": "session.started", "session": {"id": "s"}})
}

async fn drive_client(client: &mut GptLiveClient) -> Outcome {
    send_recorded_audio(client).await?;
    send_recorded_deliveries(client).await?;
    Ok(client.retire().await?)
}

async fn messages_sent() -> Result<Vec<Value>, Failure> {
    let mut server = start(&[started()]).await?;
    let (mut client, _) = connect_to(&server, "marin", false).await?;
    drive_client(&mut client).await?;
    Ok(received_after_handshake(&mut server, 8).await)
}

#[test]
fn every_recorded_python_client_message_is_sent_as_recorded() -> Outcome {
    let recorded = cases(MESSAGES, 1)?;
    same(
        &Value::Array(block_on(messages_sent())??),
        &recorded[0]["sent"],
        &recorded[0],
    );
    Ok(())
}

async fn collect_events(client: &mut GptLiveClient) -> Result<Vec<Value>, Failure> {
    let mut produced = Vec::new();
    while let Some(events) = client.next_events().await? {
        produced.extend(events_json(&events)?);
    }
    Ok(produced)
}

async fn events_until_closed() -> Result<Vec<Value>, Failure> {
    let heard = json!({"type": "session.input_transcript.delta", "delta": "oi"});
    let closed = json!({"type": "session.closed", "usage": {"tokens": 2}});
    let server = start(&[started(), heard, closed]).await?;
    let (mut client, _) = connect_to(&server, "marin", false).await?;
    client.send_audio(&[1, 0]).await?;
    collect_events(&mut client).await
}

#[test]
fn server_events_reach_the_caller_until_the_session_closes() -> Outcome {
    let produced = block_on(events_until_closed())??;
    let expected = [
        json!({"kind": "user_said", "text": "oi"}),
        json!({"kind": "usage", "details": {"tokens": 2}}),
    ];
    assert_eq!(produced, expected);
    Ok(())
}
