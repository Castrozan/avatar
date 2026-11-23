use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use super::machine::{Machine, TestResult, text};

const EMPTY_LISTING: &[u8] =
    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]";

fn answer(mut stream: TcpStream) {
    let mut request = [0_u8; 4096];
    let _ = stream.read(&mut request);
    let _ = stream.write_all(EMPTY_LISTING);
}

fn serve_empty_listings(listener: &TcpListener) {
    while let Ok((stream, _)) = listener.accept() {
        answer(stream);
    }
}

fn answer_every_request_with_an_empty_listing(listener: TcpListener) {
    thread::spawn(move || serve_empty_listings(&listener));
}

#[test]
fn an_unreachable_devtools_port_fails_before_any_encoder_starts() -> TestResult {
    let machine = Machine::with_tools(&[("ffmpeg", r#"echo started >> "${0%/*}/../log""#)])?;
    let unused = TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();
    let output = machine
        .command(&["camera", "--fps", "30"])
        .env("CDP_PORT", unused.to_string())
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).starts_with("avatar: "));
    assert_eq!(machine.logged(), "");
    Ok(())
}

#[test]
fn a_browser_that_shows_no_renderer_tab_after_opening_one_is_reported() -> TestResult {
    let machine = Machine::with_tools(&[("ffmpeg", r#"echo started >> "${0%/*}/../log""#)])?;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    answer_every_request_with_an_empty_listing(listener);
    let output = machine
        .command(&["camera"])
        .env("CDP_PORT", port.to_string())
        .env("AVATAR_RENDERER_PORT", "3999")
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("ChatVRM tab not found"));
    assert!(text(&output.stderr).contains("port 3999"));
    assert_eq!(machine.logged(), "");
    Ok(())
}
