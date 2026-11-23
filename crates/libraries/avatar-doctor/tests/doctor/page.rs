use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::thread;
use std::time::Duration;

use avatar_doctor::doctor::{LocalPageProbe, PageProbe};

fn serve_once(status_line: &'static str) -> Result<u16, std::io::Error> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    let port = listener.local_addr()?.port();
    thread::spawn(move || answer_one(&listener, status_line));
    Ok(port)
}

fn answer_one(listener: &TcpListener, status_line: &str) {
    if let Ok((mut stream, _)) = listener.accept() {
        let mut request = [0_u8; 1024];
        let _ignored = stream.read(&mut request);
        let _ignored =
            stream.write_all(format!("{status_line}\r\nContent-Length: 0\r\n\r\n").as_bytes());
    }
}

fn probe() -> LocalPageProbe {
    LocalPageProbe::with_timeout(Duration::from_secs(3))
}

#[test]
fn a_page_answering_200_is_serving() -> Result<(), std::io::Error> {
    let port = serve_once("HTTP/1.1 200 OK")?;
    assert!(probe().serves(port));
    Ok(())
}

#[test]
fn a_page_answering_an_error_is_not_serving() -> Result<(), std::io::Error> {
    let port = serve_once("HTTP/1.1 503 Service Unavailable")?;
    assert!(!probe().serves(port));
    Ok(())
}

#[test]
fn a_closed_port_is_not_serving() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))?;
    let port = listener.local_addr()?.port();
    drop(listener);
    assert!(!probe().serves(port));
    Ok(())
}
