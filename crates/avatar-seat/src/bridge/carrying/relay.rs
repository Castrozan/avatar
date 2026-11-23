use std::io;

use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

use crate::bridge::carrying::frames::play_binary_frames;
use crate::bridge::carrying::race::{until_first_finishes, until_stopped};
use crate::bridge::commands::{
    VOICE_CHUNK_BYTES, microphone_playback_command, voice_capture_command,
};
use crate::bridge::tools::{AudioTools, Capture, Playback};

pub(crate) type Incoming = SplitStream<WebSocket>;
pub(crate) type Started<Tools> = (
    <Tools as AudioTools>::Playback,
    <Tools as AudioTools>::Capture,
);

pub(crate) async fn write_queued(
    mut sink: SplitSink<WebSocket, Message>,
    mut queued: UnboundedReceiver<Message>,
) {
    while let Some(message) = queued.recv().await
        && sink.send(message).await.is_ok()
    {}
    let _ = sink.close().await;
}

pub(crate) async fn relay_voice(capture: &mut impl Capture, outgoing: &UnboundedSender<Message>) {
    while let Ok(chunk) = capture.read(VOICE_CHUNK_BYTES).await
        && !chunk.is_empty()
        && outgoing.send(Message::Binary(chunk.into())).is_ok()
    {}
}

pub(crate) fn open_writer(
    socket: WebSocket,
) -> (UnboundedSender<Message>, JoinHandle<()>, Incoming) {
    let (sink, incoming) = socket.split();
    let (outgoing, queued) = mpsc::unbounded_channel();
    (outgoing, tokio::spawn(write_queued(sink, queued)), incoming)
}

pub(crate) async fn start_tools<Tools: AudioTools>(tools: &Tools) -> io::Result<Started<Tools>> {
    let mut microphone = tools.playback(microphone_playback_command()).await?;
    let voice = tools.capture(voice_capture_command()).await;
    if voice.is_err() {
        microphone.end().await;
    }
    Ok((microphone, voice?))
}

pub(crate) async fn relay_until_over<Tools: AudioTools>(
    stopping: tokio::sync::watch::Receiver<bool>,
    channels: (Incoming, &UnboundedSender<Message>),
    tools: &mut Started<Tools>,
) {
    let (incoming, outgoing) = channels;
    let relays = until_first_finishes(
        play_binary_frames(incoming, &mut tools.0),
        relay_voice(&mut tools.1, outgoing),
    );
    until_stopped(stopping, relays).await;
}
