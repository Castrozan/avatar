use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

pub(super) struct MockServer {
    pub(super) port: u16,
    requests: Arc<AtomicUsize>,
}

impl MockServer {
    pub(super) fn serving(body: &'static [u8]) -> std::io::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        let port = listener.local_addr()?.port();
        let requests = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&requests);
        thread::spawn(move || answer_forever(&listener, body, &counter));
        Ok(Self { port, requests })
    }

    pub(super) fn requests(&self) -> usize {
        self.requests.load(Ordering::SeqCst)
    }
}

fn answer_forever(listener: &TcpListener, body: &[u8], counter: &AtomicUsize) {
    for stream in listener.incoming().flatten() {
        counter.fetch_add(1, Ordering::SeqCst);
        let _ = reply(stream, body);
    }
}

fn reply(mut stream: TcpStream, body: &[u8]) -> std::io::Result<()> {
    let mut request = [0_u8; 1024];
    let _ = stream.read(&mut request)?;
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body)
}
