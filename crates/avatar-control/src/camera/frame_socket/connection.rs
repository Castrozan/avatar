use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::{WebSocketStream, accept_hdr_async_with_config};

use super::origin::OriginCheck;

pub(super) async fn receive_frames(
    stream: TcpStream,
    allowed_origins: Vec<String>,
    frame_byte_length: usize,
    frames: UnboundedSender<Vec<u8>>,
) {
    let limits = WebSocketConfig::default().max_message_size(Some(frame_byte_length));
    let Ok(socket) =
        accept_hdr_async_with_config(stream, OriginCheck::new(allowed_origins), Some(limits)).await
    else {
        return;
    };
    relay_binary_frames(socket, frame_byte_length, frames).await;
}

async fn relay_binary_frames(
    mut socket: WebSocketStream<TcpStream>,
    frame_byte_length: usize,
    frames: UnboundedSender<Vec<u8>>,
) {
    while let Some(Ok(message)) = socket.next().await
        && forward_frame(message, frame_byte_length, &frames)
    {}
}

fn forward_frame(
    message: Message,
    frame_byte_length: usize,
    frames: &UnboundedSender<Vec<u8>>,
) -> bool {
    exact_binary_frame(message, frame_byte_length).is_none_or(|frame| frames.send(frame).is_ok())
}

fn exact_binary_frame(message: Message, frame_byte_length: usize) -> Option<Vec<u8>> {
    match message {
        Message::Binary(bytes) if bytes.len() == frame_byte_length => Some(bytes.to_vec()),
        _ => None,
    }
}
