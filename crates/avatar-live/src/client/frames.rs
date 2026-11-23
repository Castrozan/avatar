use futures_util::StreamExt;
use futures_util::future::ready;
use futures_util::stream::SplitStream;
use tokio_tungstenite::tungstenite::Error as SocketError;
use tokio_tungstenite::tungstenite::Message;

use super::{ClientError, GptLiveClient, Socket};
use crate::events::LiveEvent;
use crate::translation::{EventTranslator, Translation};

enum Frame {
    Events(Translation),
    Ended,
}

pub(super) type Events = Result<Option<Vec<LiveEvent>>, ClientError>;

fn frame_of(
    received: Result<Message, SocketError>,
    translator: &mut EventTranslator,
) -> Result<Option<Frame>, ClientError> {
    match received {
        Ok(Message::Text(message)) => translated(translator, message.as_str()),
        Ok(Message::Close(_)) | Err(SocketError::ConnectionClosed) => Ok(Some(Frame::Ended)),
        Err(failure) => Err(failure.into()),
        Ok(_) => Ok(None),
    }
}

fn translated(
    translator: &mut EventTranslator,
    message: &str,
) -> Result<Option<Frame>, ClientError> {
    let translation = translator
        .translate(message)
        .map_err(|failure| ClientError(failure.0))?;
    Ok(Some(Frame::Events(translation)))
}

async fn next_frame(
    stream: &mut SplitStream<Socket>,
    translator: &mut EventTranslator,
) -> Result<Frame, ClientError> {
    let mut frames =
        stream.filter_map(|received| ready(frame_of(received, translator).transpose()));
    frames.next().await.unwrap_or(Ok(Frame::Ended))
}

fn settle(client: &mut GptLiveClient, frame: Frame) -> Option<Vec<LiveEvent>> {
    let (ended, events) = match frame {
        Frame::Ended => (true, None),
        Frame::Events(translation) => (translation.closed, Some(translation.events)),
    };
    client.ended = ended;
    events
}

pub(super) async fn next_events(client: &mut GptLiveClient) -> Events {
    if client.ended {
        return Ok(None);
    }
    let frame = next_frame(&mut client.stream, &mut client.translator).await?;
    Ok(settle(client, frame))
}
