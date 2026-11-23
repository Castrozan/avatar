use std::io::{self, BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

const PAGE_TIMEOUT: Duration = Duration::from_secs(3);
const ROOT_REQUEST: &[u8] = b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
const OK_STATUS: u16 = 200;

pub trait PageProbe {
    fn serves(&self, port: u16) -> bool;
}

pub struct LocalPageProbe {
    timeout: Duration,
}

impl LocalPageProbe {
    #[must_use]
    pub const fn with_timeout(timeout: Duration) -> Self {
        Self { timeout }
    }

    fn status_of(&self, port: u16) -> io::Result<u16> {
        let mut stream = connect(port, self.timeout)?;
        stream.write_all(ROOT_REQUEST)?;
        let mut status_line = String::new();
        BufReader::new(stream).read_line(&mut status_line)?;
        parse_status(&status_line)
    }
}

fn connect(port: u16, timeout: Duration) -> io::Result<TcpStream> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let stream = TcpStream::connect_timeout(&address, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    Ok(stream)
}

fn parse_status(status_line: &str) -> io::Result<u16> {
    status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no HTTP status"))
}

impl Default for LocalPageProbe {
    fn default() -> Self {
        Self::with_timeout(PAGE_TIMEOUT)
    }
}

impl PageProbe for LocalPageProbe {
    fn serves(&self, port: u16) -> bool {
        self.status_of(port).is_ok_and(|status| status == OK_STATUS)
    }
}
