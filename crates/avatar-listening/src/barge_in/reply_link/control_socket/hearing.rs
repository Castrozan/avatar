use futures_util::StreamExt;
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

use super::{Incoming, LinkResult, ReplyLink};
use crate::barge_in::reply_link::ServerNotice;

pub(super) async fn hear_until_closed(link: &ReplyLink, mut stream: Incoming) -> LinkResult<()> {
    while let Some(message) = stream.next().await {
        hear_message(link, message?)?;
    }
    Ok(())
}

fn hear_message(link: &ReplyLink, message: Message) -> LinkResult<()> {
    if let Some(notice) = notice_from(message)? {
        link.speaking_state().hear_notice(&notice);
    }
    Ok(())
}

fn notice_from(message: Message) -> LinkResult<Option<ServerNotice>> {
    let Message::Text(text) = message else {
        return Ok(None);
    };
    let value: Value = serde_json::from_str(text.as_str())?;
    Ok(Some(
        ServerNotice::from_message(&value)
            .ok_or("speechActivity notice without a speaking flag")?,
    ))
}
