use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

pub(crate) struct MockServer {
    pub(crate) port: u16,
    requests: Arc<Mutex<Vec<String>>>,
}

impl MockServer {
    pub(crate) fn start(status: u16, body: &'static str) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&requests);
        thread::spawn(move || serve(&listener, status, body, &seen));
        Ok(Self { port, requests })
    }

    pub(crate) fn paths(&self) -> Vec<String> {
        let paths = self.requests.lock();
        paths.map(|paths| paths.clone()).unwrap_or_default()
    }
}

fn serve(listener: &TcpListener, status: u16, body: &str, seen: &Mutex<Vec<String>>) {
    for stream in listener.incoming().flatten() {
        answer(stream, status, body, seen);
    }
}

fn request_path(stream: &mut TcpStream) -> String {
    let mut buffer = vec![0_u8; 4096];
    let length = stream.read(&mut buffer).unwrap_or(0);
    let text = String::from_utf8_lossy(&buffer[..length]);
    text.split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_owned()
}

fn response(status: u16, body: &str) -> String {
    let length = body.len();
    format!("HTTP/1.1 {status} X\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n{body}")
}

fn answer(mut stream: TcpStream, status: u16, body: &str, seen: &Mutex<Vec<String>>) {
    let path = request_path(&mut stream);
    if let Ok(mut paths) = seen.lock() {
        paths.push(path);
    }
    let _ = stream.write_all(response(status, body).as_bytes());
}

pub(crate) fn closed_port() -> std::io::Result<u16> {
    Ok(TcpListener::bind("127.0.0.1:0")?.local_addr()?.port())
}
