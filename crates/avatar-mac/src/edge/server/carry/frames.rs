use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::frame::Frame;

const NORMAL_CLOSURE: u16 = 1000;

enum Step {
    Data(Frame),
    Skip,
    End,
}

fn frame_of(message: Message) -> Option<Frame> {
    match message {
        Message::Binary(bytes) => Some(Frame::Binary(bytes.to_vec())),
        Message::Text(text) => Some(Frame::Text(text.as_str().to_owned())),
        _ => None,
    }
}

fn step_of(received: Result<Message, axum::Error>) -> Step {
    match received {
        Ok(Message::Close(_)) | Err(_) => Step::End,
        Ok(message) => frame_of(message).map_or(Step::Skip, Step::Data),
    }
}

async fn close_with(mut socket: WebSocket, code: u16, reason: &'static str) {
    let frame = CloseFrame {
        code,
        reason: reason.into(),
    };
    let _ = socket.send(Message::Close(Some(frame))).await;
}

pub(crate) async fn refuse(socket: WebSocket, code: u16, reason: &'static str) {
    close_with(socket, code, reason).await;
}

pub(crate) async fn finish(socket: WebSocket) {
    close_with(socket, NORMAL_CLOSURE, "").await;
}

pub(crate) async fn next_frame(socket: &mut WebSocket) -> Option<Frame> {
    loop {
        match step_of(socket.recv().await?) {
            Step::Data(frame) => break Some(frame),
            Step::End => break None,
            Step::Skip => (),
        }
    }
}

pub(crate) async fn forward_messages(
    mut receiver: UnboundedReceiver<Message>,
    mut sink: SplitSink<WebSocket, Message>,
) {
    while let Some(message) = receiver.recv().await
        && sink.send(message).await.is_ok()
    {}
}

pub(crate) async fn until_closed(mut socket: SplitStream<WebSocket>) {
    while socket.next().await.is_some_and(|message| message.is_ok()) {}
}
