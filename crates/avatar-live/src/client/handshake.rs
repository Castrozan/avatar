use futures_util::StreamExt;
use futures_util::future::ready;
use futures_util::stream::SplitStream;
use tokio_tungstenite::connect_async_with_config;
use tokio_tungstenite::tungstenite::Error as SocketError;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

use super::{ClientError, GptLiveClient, Outcome, Socket};
use crate::ordered_json::{self, field, text};
use crate::session::session_start;

pub struct SessionRequest<'a> {
    pub endpoint: &'a str,
    pub api_key: &'a str,
    pub speaking_voice: &'a str,
    pub taking_over: bool,
}

fn upgrade_request(request: &SessionRequest<'_>) -> Result<Request, ClientError> {
    let mut upgrade = request.endpoint.into_client_request()?;
    let bearer = format!("Bearer {}", request.api_key.trim());
    let header = bearer
        .parse()
        .map_err(|_| ClientError("invalid api key".to_owned()))?;
    upgrade.headers_mut().insert("Authorization", header);
    Ok(upgrade)
}

async fn open_socket(upgrade: Request) -> Result<Socket, ClientError> {
    let configuration = WebSocketConfig::default()
        .max_message_size(None)
        .max_frame_size(None);
    let (socket, _) = connect_async_with_config(upgrade, Some(configuration), false).await?;
    Ok(socket)
}

fn session_id(event: &avatar_transcript::json::Json) -> Option<String> {
    let session = field(event, "session")?;
    text(field(session, "id")).map(str::to_owned)
}

struct Started {
    session: Option<String>,
}

fn handshake_of(message: &str) -> Outcome<Option<Started>> {
    let event = ordered_json::parse(message).map_err(|failure| ClientError(failure.to_string()))?;
    match text(field(&event, "type")) {
        Some("session.started") => Ok(Some(Started {
            session: session_id(&event),
        })),
        Some("error") => Err(ClientError(format!("{:?}", field(&event, "error")))),
        _ => Ok(None),
    }
}

fn started_of(received: Result<Message, SocketError>) -> Outcome<Option<Started>> {
    match received? {
        Message::Text(message) => handshake_of(message.as_str()),
        _ => Ok(None),
    }
}

fn closed_early() -> Outcome<Started> {
    Err(ClientError(
        "connection closed before session.started".to_owned(),
    ))
}

async fn await_started(stream: &mut SplitStream<Socket>) -> Outcome<Option<String>> {
    let mut started = stream.filter_map(|received| ready(started_of(received).transpose()));
    let first = started.next().await.unwrap_or_else(closed_early);
    first.map(|found| found.session)
}

async fn handshake(request: &SessionRequest<'_>) -> Outcome<(GptLiveClient, Option<String>)> {
    let mut client = GptLiveClient::new(open_socket(upgrade_request(request)?).await?);
    client
        .send(&session_start(request.speaking_voice, request.taking_over))
        .await?;
    let session = await_started(&mut client.stream).await?;
    Ok((client, session))
}

impl GptLiveClient {
    pub async fn connect(
        request: &SessionRequest<'_>,
    ) -> Result<(Self, Option<String>), ClientError> {
        handshake(request).await
    }
}
