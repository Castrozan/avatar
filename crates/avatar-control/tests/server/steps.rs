use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::error::Error;

use serde_json::Value;

use crate::TestResult;
use crate::client::{Socket, close_and_wait, connect, next_json, send_text};
use crate::ensure;

pub(crate) async fn replay_step(
    url: &str,
    sockets: &mut HashMap<u64, Socket>,
    step: &Value,
    expectation: &Value,
) -> TestResult {
    let client = step["client"].as_u64().unwrap_or_default();
    match step["op"].as_str() {
        Some("receive") => receive_step(url, sockets, client, step, expectation).await,
        Some("disconnect") => disconnect_step(sockets, client).await,
        other => Err(format!("unsupported fixture op {other:?}").into()),
    }
}

async fn receive_step(
    url: &str,
    sockets: &mut HashMap<u64, Socket>,
    client: u64,
    step: &Value,
    expectation: &Value,
) -> TestResult {
    let frame = step["frame"].as_str().unwrap_or_default();
    send_text(socket_for(url, sockets, client).await?, frame).await?;
    for entry in expectation["sent"].as_array().into_iter().flatten() {
        expect_sent(sockets, entry).await?;
    }
    Ok(())
}

async fn expect_sent(sockets: &mut HashMap<u64, Socket>, entry: &Value) -> TestResult {
    let target = entry[0].as_u64().unwrap_or_default();
    match sockets.get_mut(&target) {
        Some(socket) => check_received(socket, &entry[1]).await,
        None => Ok(()),
    }
}

async fn check_received(socket: &mut Socket, expected: &Value) -> TestResult {
    let received = next_json(socket).await?;
    ensure(received == *expected, || {
        format!("sent mismatch: {received}")
    })
}

async fn disconnect_step(sockets: &mut HashMap<u64, Socket>, client: u64) -> TestResult {
    if let Some(mut socket) = sockets.remove(&client) {
        close_and_wait(&mut socket).await?;
    }
    Ok(())
}

async fn socket_for<'sockets>(
    url: &str,
    sockets: &'sockets mut HashMap<u64, Socket>,
    client: u64,
) -> Result<&'sockets mut Socket, Box<dyn Error>> {
    match sockets.entry(client) {
        Entry::Occupied(slot) => Ok(slot.into_mut()),
        Entry::Vacant(slot) => Ok(slot.insert(connect(url).await?)),
    }
}
