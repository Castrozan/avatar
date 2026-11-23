use axum::extract::ws::{Message as ServerMessage, WebSocket};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{broadcast, mpsc};

use super::MockCommand;

pub(super) async fn serve(
    socket: WebSocket,
    received: mpsc::UnboundedSender<String>,
    commands: broadcast::Receiver<MockCommand>,
) {
    let (sink, stream) = socket.split();
    tokio::spawn(record_inbound(stream, received));
    relay_outbound(sink, commands).await;
}

async fn record_inbound(
    mut stream: SplitStream<WebSocket>,
    received: mpsc::UnboundedSender<String>,
) {
    while let Some(Ok(message)) = stream.next().await
        && let ServerMessage::Text(text) = message
    {
        received.send(text.to_string()).ok();
    }
}

async fn relay_outbound(
    mut sink: SplitSink<WebSocket, ServerMessage>,
    mut commands: broadcast::Receiver<MockCommand>,
) {
    while let Ok(command) = commands.recv().await
        && relay_one(&mut sink, command).await
    {}
}

async fn relay_one(sink: &mut SplitSink<WebSocket, ServerMessage>, command: MockCommand) -> bool {
    let closing = matches!(command, MockCommand::Close);
    let delivered = sink.send(outgoing_message(command)).await.is_ok();
    delivered && !closing
}

fn outgoing_message(command: MockCommand) -> ServerMessage {
    match command {
        MockCommand::Notice(notice) => ServerMessage::Text(notice.into()),
        MockCommand::Close => ServerMessage::Close(None),
    }
}
