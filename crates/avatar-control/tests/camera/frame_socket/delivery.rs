use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use super::super::support::TestResult;
use super::client::{
    ALLOWED_ORIGIN, FRAME_BYTE_LENGTH, WAIT, accepted_client, connect, expect_next, pattern_frame,
    send_binary,
};

#[tokio::test]
async fn exact_length_binary_frame_is_delivered_byte_for_byte() -> TestResult {
    let (mut server, mut client) = accepted_client().await?;
    send_binary(&mut client, pattern_frame()).await?;
    expect_next(&mut server, pattern_frame()).await
}

#[tokio::test]
async fn all_ones_and_all_zero_frames_arrive_in_order() -> TestResult {
    let (mut server, mut client) = accepted_client().await?;
    send_binary(&mut client, vec![255; FRAME_BYTE_LENGTH]).await?;
    send_binary(&mut client, vec![0; FRAME_BYTE_LENGTH]).await?;
    expect_next(&mut server, vec![255; FRAME_BYTE_LENGTH]).await?;
    expect_next(&mut server, vec![0; FRAME_BYTE_LENGTH]).await
}

#[tokio::test]
async fn short_binary_frame_is_dropped() -> TestResult {
    let (mut server, mut client) = accepted_client().await?;
    send_binary(&mut client, vec![9; FRAME_BYTE_LENGTH - 1]).await?;
    send_binary(&mut client, pattern_frame()).await?;
    expect_next(&mut server, pattern_frame()).await
}

#[tokio::test]
async fn empty_binary_frame_is_dropped() -> TestResult {
    let (mut server, mut client) = accepted_client().await?;
    send_binary(&mut client, Vec::new()).await?;
    send_binary(&mut client, pattern_frame()).await?;
    expect_next(&mut server, pattern_frame()).await
}

#[tokio::test]
async fn text_message_of_frame_length_is_dropped() -> TestResult {
    let (mut server, mut client) = accepted_client().await?;
    client
        .send(Message::Text("x".repeat(FRAME_BYTE_LENGTH).into()))
        .await?;
    send_binary(&mut client, pattern_frame()).await?;
    expect_next(&mut server, pattern_frame()).await
}

#[tokio::test]
async fn frames_from_two_connections_are_each_delivered() -> TestResult {
    let (mut server, mut first) = accepted_client().await?;
    send_binary(&mut first, vec![1; FRAME_BYTE_LENGTH]).await?;
    expect_next(&mut server, vec![1; FRAME_BYTE_LENGTH]).await?;
    let mut second = connect(server.address, Some(ALLOWED_ORIGIN)).await?;
    send_binary(&mut second, vec![7; FRAME_BYTE_LENGTH]).await?;
    expect_next(&mut server, vec![7; FRAME_BYTE_LENGTH]).await
}

#[tokio::test]
async fn oversized_message_closes_its_connection_and_is_not_delivered() -> TestResult {
    let (mut server, mut client) = accepted_client().await?;
    send_binary(&mut client, vec![3; FRAME_BYTE_LENGTH + 1]).await?;
    let outcome = tokio::time::timeout(WAIT, client.next()).await?;
    assert!(!matches!(outcome, Some(Ok(Message::Binary(_)))));
    let mut next_client = connect(server.address, Some(ALLOWED_ORIGIN)).await?;
    send_binary(&mut next_client, pattern_frame()).await?;
    expect_next(&mut server, pattern_frame()).await
}
