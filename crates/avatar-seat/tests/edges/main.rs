mod process_tests;

use process_tests::{Outcome, tool};

use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

use avatar_seat::bridge::{AudioTools, Capture, Playback, SpawnedTools};
use avatar_seat::edges::{HttpProbe, SeatProbe};
use serde_json::json;

fn serve_health_once(listener: &TcpListener) {
    let reply = b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n{\"attached\": true}";
    let accepted = listener.accept();
    let _ = accepted.map(|(mut stream, _)| {
        stream
            .read(&mut [0; 512])
            .and_then(|_| stream.write_all(reply))
    });
}

#[test]
fn the_probe_reads_the_health_of_a_local_seat() -> Outcome<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    let server = std::thread::spawn(move || serve_health_once(&listener));
    assert_eq!(HttpProbe { port }.health(), Some(json!({"attached": true})));
    server.join().map_err(|_| "server panicked")?;
    Ok(())
}

#[test]
fn the_probe_reads_nothing_from_a_closed_port() -> Outcome<()> {
    let port = TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();
    assert_eq!(HttpProbe { port }.health(), None);
    Ok(())
}

async fn wait_while_false(mut condition: impl FnMut() -> bool) {
    while !condition() {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

async fn eventually_async(condition: impl FnMut() -> bool) -> bool {
    tokio::time::timeout(Duration::from_secs(5), wait_while_false(condition))
        .await
        .is_ok()
}

#[tokio::test]
async fn the_spawned_playback_feeds_a_real_process() -> Outcome<()> {
    let directory = tempfile::tempdir()?;
    let sink = directory.path().join("heard");
    let player = tool(directory.path(), &format!("exec cat > {}", sink.display()))?;
    let mut playback = SpawnedTools.playback(vec![player]).await?;
    playback.write(b"frames").await?;
    assert!(eventually_async(|| std::fs::read(&sink).is_ok_and(|heard| heard == b"frames")).await);
    playback.end().await;
    Ok(())
}

#[tokio::test]
async fn the_spawned_capture_reads_a_real_process() -> Outcome<()> {
    let directory = tempfile::tempdir()?;
    let recorder = tool(directory.path(), "printf voice; exec sleep 30")?;
    let mut capture = SpawnedTools.capture(vec![recorder]).await?;
    assert_eq!(capture.read(960).await?, b"voice");
    capture.end().await;
    Ok(())
}
