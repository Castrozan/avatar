use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

use serde_json::Value;

const PROBE_TIMEOUT: Duration = Duration::from_secs(1);

pub trait SeatProbe {
    fn health(&self) -> Option<Value>;
}

pub struct HttpProbe {
    pub port: u16,
}

fn fetch(port: u16) -> std::io::Result<Vec<u8>> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let mut stream = TcpStream::connect_timeout(&address, PROBE_TIMEOUT)?;
    stream.set_read_timeout(Some(PROBE_TIMEOUT))?;
    stream.write_all(b"GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")?;
    let mut reply = Vec::new();
    stream.read_to_end(&mut reply)?;
    Ok(reply)
}

fn body_of(reply: &[u8]) -> Option<&[u8]> {
    let separator = reply.windows(4).position(|window| window == b"\r\n\r\n")?;
    reply.get(separator + 4..)
}

impl SeatProbe for HttpProbe {
    fn health(&self) -> Option<Value> {
        let reply = fetch(self.port).ok()?;
        serde_json::from_slice(body_of(&reply)?).ok()
    }
}
