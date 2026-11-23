use std::sync::Arc;

use futures_util::StreamExt;
use tokio::runtime::Runtime;
use tokio_tungstenite::tungstenite::Message;

use super::connection::{Socket, open_socket, private_runtime};
use crate::client::{CALL_PATH, PORT};
use crate::failure::Failure;
use crate::frame::Frame;

const OPEN_TIMEOUT_SECONDS: u64 = 5;

pub struct CallFrames {
    runtime: Arc<Runtime>,
    socket: Socket,
    abnormal: bool,
}

fn frame_of(message: Message) -> Option<Frame> {
    match message {
        Message::Binary(bytes) => Some(Frame::Binary(bytes.to_vec())),
        Message::Text(text) => Some(Frame::Text(text.as_str().to_owned())),
        _ => None,
    }
}

impl CallFrames {
    pub fn connect(port: u16) -> Result<Self, Failure> {
        let runtime = private_runtime()?;
        let opening = open_socket(port, CALL_PATH, OPEN_TIMEOUT_SECONDS);
        let socket = runtime.block_on(opening)?;
        Ok(Self {
            runtime,
            socket,
            abnormal: false,
        })
    }

    pub fn connect_on_this_mac() -> Result<Self, Failure> {
        Self::connect(PORT)
    }

    #[must_use]
    pub const fn ended_abnormally(&self) -> bool {
        self.abnormal
    }
}

fn abnormal(frames: &mut CallFrames) -> Option<Frame> {
    frames.abnormal = true;
    None
}

fn advance(frames: &mut CallFrames) -> Option<Frame> {
    match frames.runtime.block_on(frames.socket.next())? {
        Ok(Message::Close(_)) => None,
        Ok(message) => frame_of(message).or_else(|| advance(frames)),
        Err(_) => abnormal(frames),
    }
}

impl Iterator for CallFrames {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        advance(self)
    }
}
