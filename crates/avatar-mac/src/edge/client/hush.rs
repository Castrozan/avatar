use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

use crate::client::HUSH_PATH;

fn request(port: u16, timeout: Duration) -> std::io::Result<()> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let mut stream = TcpStream::connect_timeout(&address, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    let text = format!("GET {HUSH_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    stream.write_all(text.as_bytes())?;
    stream.read_to_end(&mut Vec::new()).map(drop)
}

pub(super) fn hush_the_seat(port: u16, timeout: Duration) {
    let _ = request(port, timeout);
}
