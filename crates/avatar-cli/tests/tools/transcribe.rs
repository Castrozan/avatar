use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;

use super::{TestResult, run_isolated};

const MODELS: &str = r#"[{"id": "parakeet_v3", "installed": true}]"#;
const SPOKEN: &str = r#"{"transcript": "  hey   Voyager, check the build "}"#;

fn answer(stream: &mut TcpStream) {
    let mut buffer = vec![0_u8; 8192];
    let length = stream.read(&mut buffer).unwrap_or(0);
    let models = buffer.starts_with(b"GET /models") && length > 0;
    let body = if models { MODELS } else { SPOKEN };
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(format!("{head}{body}").as_bytes());
}

fn serve_hex(listener: &TcpListener) {
    for mut stream in listener.incoming().flatten() {
        answer(&mut stream);
    }
}

fn discover(home: &Path, port: u16) -> TestResult {
    fs::create_dir_all(home.join(".local/share/hex"))?;
    let discovery = format!(r#"{{"port": {port}, "token": "t"}}"#);
    fs::write(home.join(".local/share/hex/local-api.json"), discovery)?;
    Ok(())
}

#[test]
fn the_transcript_of_the_pcm_on_stdin_is_printed_with_a_newline() -> TestResult {
    let home = tempfile::tempdir()?;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    discover(home.path(), listener.local_addr()?.port())?;
    thread::spawn(move || serve_hex(&listener));
    let output = run_isolated(&["transcribe"], home.path(), &[1, 2].repeat(320))?;
    assert_eq!(
        String::from_utf8(output.stdout)?,
        "hey Voyager, check the build\n"
    );
    Ok(())
}

#[test]
fn an_unreachable_hex_fails_with_a_message_and_a_failing_status() -> TestResult {
    let home = tempfile::tempdir()?;
    let output = run_isolated(&["transcribe"], home.path(), b"")?;
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty() && !output.stderr.is_empty());
    Ok(())
}
