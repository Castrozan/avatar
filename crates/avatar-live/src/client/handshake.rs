use futures_util::StreamExt;
use futures_util::future::ready;
use futures_util::stream::SplitStream;
use tokio_tungstenite::connect_async_with_config;
use tokio_tungstenite::tungstenite::Error as SocketError;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, WebSocketConfig};

use super::{ClientError, LiveClient, Outcome, Socket};
use crate::ordered_json::{self, field, text};
use crate::session::initiation;

pub struct SessionRequest<'a> {
    pub url: &'a str,
    pub speaking_voice: &'a str,
    pub taking_over: bool,
    pub brief: Option<&'a str>,
}

async fn open_socket(url: &str) -> Result<Socket, ClientError> {
    let configuration = WebSocketConfig::default()
        .max_message_size(None)
        .max_frame_size(None);
    let upgrade = url.into_client_request()?;
    let (socket, _) = connect_async_with_config(upgrade, Some(configuration), false).await?;
    Ok(socket)
}

fn conversation_id(event: &avatar_transcript::json::Json) -> Option<String> {
    let metadata = field(event, "conversation_initiation_metadata_event")?;
    text(field(metadata, "conversation_id")).map(str::to_owned)
}

struct Started {
    conversation: Option<String>,
}

fn handshake_of(message: &str) -> Outcome<Option<Started>> {
    let event = ordered_json::parse(message).map_err(|failure| ClientError(failure.to_string()))?;
    match text(field(&event, "type")) {
        Some("conversation_initiation_metadata") => Ok(Some(Started {
            conversation: conversation_id(&event),
        })),
        Some("client_error") => Err(ClientError(format!("{:?}", field(&event, "error_event")))),
        _ => Ok(None),
    }
}

fn refused(frame: Option<&CloseFrame>) -> ClientError {
    let reason = frame.map_or_else(String::new, |close| {
        format!(": {} {}", close.code, close.reason)
    });
    ClientError(format!("conversation closed before it started{reason}"))
}

fn started_of(received: Result<Message, SocketError>) -> Outcome<Option<Started>> {
    match received? {
        Message::Text(message) => handshake_of(message.as_str()),
        Message::Close(frame) => Err(refused(frame.as_ref())),
        _ => Ok(None),
    }
}

async fn await_started(stream: &mut SplitStream<Socket>) -> Outcome<Option<String>> {
    let mut started = stream.filter_map(|received| ready(started_of(received).transpose()));
    let first = started.next().await.unwrap_or_else(|| Err(refused(None)));
    first.map(|found| found.conversation)
}

async fn handshake(request: &SessionRequest<'_>) -> Outcome<(LiveClient, Option<String>)> {
    let mut client = LiveClient::new(open_socket(request.url).await?);
    let start = initiation(request.speaking_voice, request.taking_over, request.brief);
    client.send(&start).await?;
    let conversation = await_started(&mut client.stream).await?;
    Ok((client, conversation))
}

impl LiveClient {
    pub async fn connect(
        request: &SessionRequest<'_>,
    ) -> Result<(Self, Option<String>), ClientError> {
        handshake(request).await
    }
}
