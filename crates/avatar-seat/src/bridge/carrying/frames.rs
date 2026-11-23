use std::future::ready;

use axum::body::Bytes;
use axum::extract::ws::Message;
use futures_util::{Stream, StreamExt};

use crate::bridge::tools::Playback;

fn binary_of(frame: Result<Message, axum::Error>) -> Option<Bytes> {
    match frame {
        Ok(Message::Binary(bytes)) => Some(bytes),
        _ => None,
    }
}

pub(crate) async fn play_binary_frames<Frames>(frames: Frames, playback: &mut impl Playback)
where
    Frames: Stream<Item = Result<Message, axum::Error>> + Unpin,
{
    let open = frames.take_while(|frame| ready(frame.is_ok()));
    let mut binary = open.filter_map(|frame| ready(binary_of(frame)));
    while let Some(bytes) = binary.next().await
        && playback.write(&bytes).await.is_ok()
    {}
}
