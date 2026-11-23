use std::time::Duration;

use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message;

use crate::harness::*;
use crate::pages::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

async fn binary_sizes(
    page: &mut Page,
    count: usize,
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let mut sizes = Vec::new();
    while sizes.len() < count {
        sizes.push(next_binary(page).await?.len());
    }
    Ok(sizes)
}

async fn chunks_before_the_notice(page: &mut Page) -> Result<usize, Box<dyn std::error::Error>> {
    let mut chunks = 1;
    while let Message::Binary(_) = next_message(page).await? {
        chunks += 1;
    }
    Ok(chunks)
}

#[tokio::test]
async fn speech_reaches_the_page_in_twenty_millisecond_chunks() -> TestResult {
    let seat = start_seat(Duration::from_secs(30)).await?;
    let mut page = attached_page(seat.address).await?;
    let mut speech = connect_allowed(seat.address, "/speech").await?;
    speech
        .send(Message::Binary(vec![7; 960 * 3 + 101].into()))
        .await?;
    assert_eq!(binary_sizes(&mut page, 4).await?, [960, 960, 960, 100]);
    let closing = close_of(&next_message(&mut speech).await?);
    assert_eq!(closing, Some((1000, String::new())));
    Ok(())
}

const RATE_REFUSAL: &str = "The seat plays 24 kHz audio";
const BUSY_REFUSAL: &str = "Another page carries the call";

#[tokio::test]
async fn hushing_tells_the_page_and_stops_the_speech() -> TestResult {
    let seat = start_seat(Duration::from_secs(30)).await?;
    let mut page = attached_page(seat.address).await?;
    let mut speech = connect_allowed(seat.address, "/speech").await?;
    send_binary(&mut speech, vec![0; 144_000]).await?;
    next_binary(&mut page).await?;
    assert_eq!(status_of(seat.address, "/hush", "").await?, 200);
    assert!(chunks_before_the_notice(&mut page).await? < 40);
    assert_eq!(closed_with(&mut speech).await?, Some((1000, String::new())));
    Ok(())
}

#[tokio::test]
async fn a_stream_must_announce_the_voice_rate() -> TestResult {
    let seat = start_seat(Duration::from_secs(30)).await?;
    let mut stream = connect_allowed(seat.address, "/stream").await?;
    announce(&mut stream, 16000).await?;
    assert_eq!(
        closed_with(&mut stream).await?,
        Some((1003, RATE_REFUSAL.to_owned()))
    );
    Ok(())
}

#[tokio::test]
async fn a_stream_forwards_whole_samples_to_the_page() -> TestResult {
    let seat = start_seat(Duration::from_secs(30)).await?;
    let mut page = attached_page(seat.address).await?;
    let mut stream = connect_allowed(seat.address, "/stream").await?;
    announce(&mut stream, 24000).await?;
    send_binary(&mut stream, vec![5; 5]).await?;
    assert_eq!(next_binary(&mut page).await?, vec![5; 4]);
    Ok(())
}

#[tokio::test]
async fn the_meeting_page_audio_reaches_every_capturer() -> TestResult {
    let seat = start_seat(Duration::from_secs(30)).await?;
    let mut capturer = connect_allowed(seat.address, "/call").await?;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let mut meeting = connect_allowed(seat.address, "/meeting").await?;
    meeting.send(Message::Binary(vec![9, 9].into())).await?;
    meeting.send(Message::text("ignored")).await?;
    assert_eq!(next_binary(&mut capturer).await?, vec![9, 9]);
    Ok(())
}

#[tokio::test]
async fn a_second_meeting_page_is_refused_until_the_first_leaves() -> TestResult {
    let seat = start_seat(Duration::from_secs(30)).await?;
    let mut first = connect_allowed(seat.address, "/meeting").await?;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let mut second = connect_allowed(seat.address, "/meeting").await?;
    assert_eq!(
        closed_with(&mut second).await?,
        Some((1013, BUSY_REFUSAL.to_owned()))
    );
    first.close(None).await?;
    Ok(())
}
