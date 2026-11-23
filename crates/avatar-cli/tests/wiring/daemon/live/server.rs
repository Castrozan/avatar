use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Builder;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{
    Callback, ErrorResponse, Request, Response,
};
use tokio_tungstenite::{WebSocketStream, accept_hdr_async};

use super::heard::Heard;
use super::script::replies_to;

struct Authorization(Sender<String>);

impl Callback for Authorization {
    fn on_request(self, request: &Request, response: Response) -> Result<Response, ErrorResponse> {
        let value = request.headers().get("authorization");
        let text = value.and_then(|header| header.to_str().ok());
        let _ = self.0.send(text.unwrap_or_default().to_owned());
        Ok(response)
    }
}

async fn converse(mut socket: WebSocketStream<TcpStream>, sink: Sender<Value>) {
    while let Some(Ok(Message::Text(text))) = socket.next().await {
        let message: Value = serde_json::from_str(text.as_str()).unwrap_or_default();
        let replies = replies_to(&message);
        let _ = sink.send(message);
        send_all(&mut socket, replies).await;
    }
}

async fn send_all(socket: &mut WebSocketStream<TcpStream>, replies: Vec<Value>) {
    for reply in replies {
        let _ = socket.send(Message::text(reply.to_string())).await;
    }
}

async fn accept(listener: TcpListener, headers: Sender<String>, sink: Sender<Value>) {
    let Ok((stream, _)) = listener.accept().await else {
        return;
    };
    if let Ok(socket) = accept_hdr_async(stream, Authorization(headers)).await {
        converse(socket, sink).await;
    }
}

pub(super) struct MockLive {
    pub(super) address: SocketAddr,
    headers: Receiver<String>,
    messages: Receiver<Value>,
}

async fn listen(
    address: Sender<std::io::Result<SocketAddr>>,
    headers: Sender<String>,
    sink: Sender<Value>,
) {
    match TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => {
            let _ = address.send(listener.local_addr());
            accept(listener, headers, sink).await;
        }
        Err(failure) => drop(address.send(Err(failure))),
    }
}

fn run_server(
    address: Sender<std::io::Result<SocketAddr>>,
    headers: Sender<String>,
    sink: Sender<Value>,
) {
    if let Ok(runtime) = Builder::new_current_thread().enable_all().build() {
        runtime.block_on(listen(address, headers, sink));
    }
}

impl MockLive {
    pub(super) fn start() -> std::io::Result<Self> {
        let ((header_sender, headers), (message_sender, messages)) = (channel(), channel());
        let (address_sender, address) = channel();
        thread::spawn(move || run_server(address_sender, header_sender, message_sender));
        let address = address.recv().map_err(std::io::Error::other)??;
        Ok(Self {
            address,
            headers,
            messages,
        })
    }

    pub(super) fn heard(self) -> Heard {
        Heard {
            authorization: self.headers.try_iter().next().unwrap_or_default(),
            messages: self.messages.try_iter().collect(),
        }
    }
}
