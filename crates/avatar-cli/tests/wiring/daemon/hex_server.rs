use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

const MODELS: &str = r#"[{"id":"parakeet_v3","installed":true}]"#;

fn content_length_of(header: &str) -> usize {
    let (name, value) = header.split_once(':').unwrap_or_default();
    let length = value.trim().parse().unwrap_or_default();
    if name.eq_ignore_ascii_case("content-length") {
        length
    } else {
        0
    }
}

fn headers_length(reader: &mut impl BufRead) -> usize {
    let headers = reader.lines().map_while(Result::ok);
    let lengths = headers.take_while(|line| !line.is_empty());
    lengths
        .map(|line| content_length_of(&line))
        .max()
        .unwrap_or_default()
}

fn read_request(stream: &TcpStream) -> Option<String> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).ok()?;
    let mut body = vec![0; headers_length(&mut reader)];
    reader.read_exact(&mut body).ok()?;
    Some(request_line)
}

fn body_for(request_line: &str, transcript: &str) -> String {
    if request_line.starts_with("GET /models") {
        return MODELS.to_owned();
    }
    format!(r#"{{"transcript":"{transcript}"}}"#)
}

fn respond(stream: &mut TcpStream, transcript: &str) {
    let Some(request_line) = read_request(stream) else {
        return;
    };
    let body = body_for(&request_line, transcript);
    let head = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n";
    let reply = format!("{head}Content-Length: {}\r\n\r\n{body}", body.len());
    let _ = stream.write_all(reply.as_bytes());
}

fn serve(listener: &TcpListener, transcript: &str) {
    for mut stream in listener.incoming().flatten() {
        respond(&mut stream, transcript);
    }
}

pub(super) fn serve_hex(transcript: &'static str) -> std::io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    thread::spawn(move || serve(&listener, transcript));
    Ok(port)
}
