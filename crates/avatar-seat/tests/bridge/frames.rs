use super::fakes::VOICE;
use super::harness::{Outcome, Socket};
use super::helpers::FIVE_SECONDS;
use futures_util::StreamExt;
use serde_json::Value;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

pub(crate) async fn next_frame(socket: &mut Socket) -> Outcome<Message> {
    Ok(timeout(FIVE_SECONDS, socket.next())
        .await?
        .ok_or("closed")??)
}

pub(crate) fn text_json(frame: &Message) -> Outcome<Value> {
    let Message::Text(text) = frame else {
        return Err("not text".into());
    };
    Ok(serde_json::from_str(text.as_str())?)
}

pub(crate) async fn next_text(socket: &mut Socket) -> Outcome<Value> {
    let mut frame = next_frame(socket).await?;
    while !matches!(frame, Message::Text(_)) {
        frame = next_frame(socket).await?;
    }
    text_json(&frame)
}

pub(crate) fn binary_of(frame: Message) -> Vec<u8> {
    match frame {
        Message::Binary(bytes) => bytes.to_vec(),
        _ => Vec::new(),
    }
}

pub(crate) async fn collect_voice(socket: &mut Socket) -> Outcome<Vec<u8>> {
    let mut voice = Vec::new();
    while voice.len() < VOICE.len() {
        voice.extend(binary_of(next_frame(socket).await?));
    }
    Ok(voice)
}

pub(crate) async fn drain_frames(socket: &mut Socket) {
    while socket.next().await.is_some_and(|frame| frame.is_ok()) {}
}

pub(crate) fn is_taken(frame: &Message) -> bool {
    matches!(frame, Message::Close(Some(close)) if close.code == CloseCode::Again)
}
