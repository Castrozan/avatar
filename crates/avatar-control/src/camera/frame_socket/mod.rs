mod connection;
mod origin;

use std::io;
use std::net::{Ipv4Addr, SocketAddr};

use tokio::net::TcpListener;
use tokio::sync::mpsc::UnboundedSender;

pub struct FrameSocketServer {
    listener: TcpListener,
    frame_byte_length: usize,
    allowed_origins: Vec<String>,
}

impl FrameSocketServer {
    pub async fn bind(
        port: u16,
        frame_byte_length: usize,
        allowed_origins: Vec<String>,
    ) -> io::Result<Self> {
        let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, port))).await?;
        Ok(Self {
            listener,
            frame_byte_length,
            allowed_origins,
        })
    }

    pub fn local_address(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub async fn serve(self, frames: UnboundedSender<Vec<u8>>) {
        accept_connections(
            self.listener,
            self.allowed_origins,
            self.frame_byte_length,
            frames,
        )
        .await;
    }
}

async fn accept_connections(
    listener: TcpListener,
    allowed_origins: Vec<String>,
    frame_byte_length: usize,
    frames: UnboundedSender<Vec<u8>>,
) {
    while let Ok((stream, _peer)) = listener.accept().await {
        tokio::spawn(connection::receive_frames(
            stream,
            allowed_origins.clone(),
            frame_byte_length,
            frames.clone(),
        ));
    }
}
