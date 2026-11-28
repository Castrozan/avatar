use avatar_live::client::LiveClient;
use serde_json::{Value, json};

use super::connect_to;
use super::server::{MockServer, start};
use crate::support::{Outcome, block_on, cases, events_json, same};

const MESSAGES: &str = include_str!("fixtures/client_messages.json");

type Failure = Box<dyn std::error::Error>;

async fn send_recorded_audio(client: &mut LiveClient) -> Outcome {
    client.send_audio(&[0, 1, 2, 255]).await?;
    Ok(client.send_audio(&[]).await?)
}

async fn send_recorded_deliveries(client: &mut LiveClient) -> Outcome {
    client.deliver("Checking.", false).await?;
    client.deliver("Done.", true).await?;
    Ok(client.deliver("\u{e7}\u{e3}o", true).await?)
}

async fn received_after_handshake(server: &mut MockServer, count: usize) -> Vec<Value> {
    let mut received = Vec::new();
    for _ in 0..count {
        received.push(server.received.recv().await.unwrap_or_default());
    }
    received.split_off(1)
}

fn started() -> Value {
    json!({"type": "conversation_initiation_metadata", "conversation_initiation_metadata_event": {"conversation_id": "c"}})
}

async fn drive_client(client: &mut LiveClient) -> Outcome {
    send_recorded_audio(client).await?;
    send_recorded_deliveries(client).await?;
    Ok(client.close().await?)
}

async fn messages_sent() -> Result<Vec<Value>, Failure> {
    let mut server = start(&[started()]).await?;
    let (mut client, _) = connect_to(&server, "v", false).await?;
    drive_client(&mut client).await?;
    Ok(received_after_handshake(&mut server, 6).await)
}

#[test]
fn every_recorded_client_message_is_sent_as_recorded() -> Outcome {
    let recorded = cases(MESSAGES, 1)?;
    let sent = Value::Array(block_on(messages_sent())??);
    same(&sent, &recorded[0]["sent"], &recorded[0]);
    Ok(())
}

async fn collect_events(client: &mut LiveClient) -> Result<Vec<Value>, Failure> {
    let mut produced = Vec::new();
    while let Some(events) = client.next_events().await? {
        produced.extend(events_json(&events));
    }
    Ok(produced)
}

fn conversation_until_closed() -> [Value; 5] {
    let heard =
        json!({"type": "user_transcript", "user_transcription_event": {"user_transcript": "oi"}});
    let call = json!({"type": "client_tool_call", "client_tool_call": {"tool_name": "hand_off_to_agent", "tool_call_id": "t-1", "parameters": {}, "event_id": 2}});
    let ping = json!({"type": "ping", "ping_event": {"event_id": 5, "ping_ms": 20}});
    [started(), heard, call, ping, json!("close")]
}

async fn events_and_replies() -> Result<(Vec<Value>, Vec<Value>), Failure> {
    let mut server = start(&conversation_until_closed()).await?;
    let (mut client, _) = connect_to(&server, "v", false).await?;
    let produced = collect_events(&mut client).await?;
    Ok((produced, received_after_handshake(&mut server, 3).await))
}

#[test]
fn server_events_reach_the_caller_and_get_their_replies_until_the_server_closes() -> Outcome {
    let (produced, replies) = block_on(events_and_replies())??;
    let task = json!({"kind": "task", "id": "t-1", "request": "oi"});
    assert_eq!(produced, [json!({"kind": "user_said", "text": "oi"}), task]);
    assert_eq!(replies[0]["type"], "client_tool_result");
    assert_eq!(replies[1], json!({"type": "pong", "event_id": 5}));
    Ok(())
}
