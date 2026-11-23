use std::net::SocketAddr;

use avatar_mac::client::{SeatEndpoint, SeatStream, wav_samples};
use avatar_mac::edge::client::{CallFrames, SeatClient};
use avatar_mac::failure::{Failure, FailureKind};
use avatar_mac::frame::Frame;
use futures_util::SinkExt;
use tokio::task::spawn_blocking;
use tokio_tungstenite::tungstenite::Message;

use crate::harness::{connect_allowed, start_seat};
use crate::pages::{attached_page, next_binary, next_message};

type TestResult = Result<(), Box<dyn std::error::Error>>;

async fn on_client<Output: Send + 'static>(
    address: SocketAddr,
    work: impl FnOnce(&mut SeatClient) -> Output + Send + 'static,
) -> Result<Output, Box<dyn std::error::Error>> {
    let outcome = spawn_blocking(move || {
        let mut client = SeatClient::on_port(address.port()).map_err(|failure| failure.message)?;
        Ok::<Output, String>(work(&mut client))
    });
    Ok(outcome.await??)
}

fn stream_four_bytes(client: &mut SeatClient) -> Result<(), Failure> {
    let mut stream = SeatStream::<SeatClient>::new(24000, "no page");
    let outcome = stream.send(client, &[1, 2, 3, 4]);
    stream.close();
    outcome
}

fn wait_briefly(client: &mut SeatClient) -> Result<(), Failure> {
    client.wait_until_attached(0.3, "the page is missing")
}

#[tokio::test(flavor = "multi_thread")]
async fn the_client_sees_whether_a_page_is_attached() -> TestResult {
    let seat = start_seat(std::time::Duration::from_secs(30)).await?;
    assert!(!on_client(seat.address, SeatClient::attached).await?);
    let _page = attached_page(seat.address).await?;
    assert!(on_client(seat.address, SeatClient::attached).await?);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn hushing_through_the_client_reaches_the_page() -> TestResult {
    let seat = start_seat(std::time::Duration::from_secs(30)).await?;
    let mut page = attached_page(seat.address).await?;
    on_client(seat.address, SeatClient::hush).await?;
    assert_eq!(next_message(&mut page).await?, Message::text("hush"));
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn playing_through_the_client_returns_when_the_speech_ends() -> TestResult {
    let seat = start_seat(std::time::Duration::from_secs(30)).await?;
    let mut page = attached_page(seat.address).await?;
    let played = on_client(seat.address, |client| client.play(&[3; 1000])).await?;
    assert!(played.is_ok());
    assert_eq!(next_binary(&mut page).await?.len(), 960);
    assert_eq!(next_binary(&mut page).await?.len(), 40);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn a_seat_stream_announces_its_rate_then_sends_audio() -> TestResult {
    let seat = start_seat(std::time::Duration::from_secs(30)).await?;
    let mut page = attached_page(seat.address).await?;
    let outcome = on_client(seat.address, stream_four_bytes).await?;
    assert!(outcome.is_ok());
    assert_eq!(next_binary(&mut page).await?, vec![1, 2, 3, 4]);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn a_stream_waits_for_the_page_and_times_out_with_its_message() -> TestResult {
    let seat = start_seat(std::time::Duration::from_secs(30)).await?;
    let outcome = on_client(seat.address, wait_briefly).await?;
    let failure = outcome.err().ok_or("expected a timeout")?;
    assert_eq!(
        (failure.kind, failure.message.as_str()),
        (FailureKind::Timeout, "the page is missing")
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn call_capture_yields_the_audio_the_meeting_page_carries() -> TestResult {
    let seat = start_seat(std::time::Duration::from_secs(30)).await?;
    let port = seat.address.port();
    let capturing =
        spawn_blocking(move || CallFrames::connect(port).map(|mut frames| frames.next()));
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let mut meeting = connect_allowed(seat.address, "/meeting").await?;
    meeting.send(Message::Binary(vec![8, 8].into())).await?;
    let frame = capturing.await??;
    assert_eq!(frame, Some(Frame::Binary(vec![8, 8])));
    Ok(())
}

#[test]
fn a_recording_is_reduced_to_its_samples() -> TestResult {
    let mut wav = b"RIFF\0\0\0\0WAVEfmt ".to_vec();
    wav.extend([
        16, 0, 0, 0, 1, 0, 1, 0, 0x80, 0xbb, 0, 0, 0, 0x77, 1, 0, 2, 0, 16, 0,
    ]);
    wav.extend(b"data");
    wav.extend([4, 0, 0, 0, 1, 2, 3, 4]);
    assert_eq!(wav_samples(&wav)?, vec![1, 2, 3, 4]);
    Ok(())
}
