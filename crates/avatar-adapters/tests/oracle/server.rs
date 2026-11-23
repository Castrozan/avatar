use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::{self, JoinHandle};

pub(crate) struct Server {
    pub(crate) url: String,
    handle: JoinHandle<String>,
}

impl Server {
    pub(crate) fn start(reply: &'static str) -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let url = format!("http://{}/v1/decisions", listener.local_addr()?);
        let handle = thread::spawn(move || serve(&listener, reply));
        Ok(Self { url, handle })
    }

    pub(crate) fn received(self) -> Result<String, Box<dyn Error>> {
        Ok(self.handle.join().map_err(|_| "server thread panicked")?)
    }
}

fn serve(listener: &TcpListener, reply: &str) -> String {
    let Ok((mut stream, _)) = listener.accept() else {
        return String::new();
    };
    let received = read_request(&mut stream);
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        reply.len()
    );
    let _ = stream.write_all(format!("{head}{reply}").as_bytes());
    received
}

fn read_request(stream: &mut TcpStream) -> String {
    let mut received = Vec::new();
    while !complete(&received) && read_more(stream, &mut received) {}
    String::from_utf8_lossy(&received).into_owned()
}

fn read_more(stream: &mut TcpStream, received: &mut Vec<u8>) -> bool {
    let mut buffer = [0_u8; 4096];
    let count = stream.read(&mut buffer).unwrap_or(0);
    received.extend_from_slice(&buffer[..count]);
    count > 0
}

fn complete(received: &[u8]) -> bool {
    let raw = String::from_utf8_lossy(received).to_ascii_lowercase();
    let Some((head, body)) = raw.split_once("\r\n\r\n") else {
        return false;
    };
    let prefix = "content-length: ";
    let length = head
        .lines()
        .find_map(|line| line.strip_prefix(prefix)?.parse::<usize>().ok());
    length.is_some_and(|length| body.len() >= length)
}
