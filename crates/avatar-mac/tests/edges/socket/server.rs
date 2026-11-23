use std::time::Duration;

use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message;

use crate::harness::*;
use crate::pages::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn the_page_files_health_and_unknown_paths_answer() -> TestResult {
    let seat = start_seat(Duration::from_secs(30)).await?;
    let (status, body) = http_get(seat.address, "/", "").await?;
    assert_eq!((status, body.as_str()), (200, "<html>seat</html>"));
    let (_, health) = http_get(seat.address, "/health", "").await?;
    assert_eq!(health, "{\"attached\":false,\"failure\":null}");
    assert_eq!(http_get(seat.address, "/nope", "").await?.0, 404);
    assert_eq!(http_get(seat.address, "/hush", "").await?.0, 200);
    Ok(())
}

#[tokio::test]
async fn a_failure_is_recorded_only_from_the_same_origin() -> TestResult {
    let seat = start_seat(Duration::from_secs(30)).await?;
    assert_eq!(status_of(seat.address, "/failure?no%20mic", "").await?, 403);
    let same = "Sec-Fetch-Site: same-origin\r\n";
    assert_eq!(
        status_of(seat.address, "/failure?no%20mic", same).await?,
        200
    );
    let (_, health) = http_get(seat.address, "/health", "").await?;
    assert_eq!(health, "{\"attached\":false,\"failure\":\"no mic\"}");
    Ok(())
}

#[tokio::test]
async fn a_websocket_from_another_origin_is_forbidden() -> TestResult {
    let seat = start_seat(Duration::from_secs(30)).await?;
    let refused = connect(seat.address, "/audio", "http://evil.example").await;
    assert!(refused.is_err());
    Ok(())
}

#[tokio::test]
async fn the_page_audio_reaches_the_output_and_leaving_ends_the_server() -> TestResult {
    let seat = start_seat(Duration::from_secs(30)).await?;
    let mut page = attached_page(seat.address).await?;
    page.send(Message::Binary(vec![1, 2, 3].into())).await?;
    page.send(Message::text("ignored")).await?;
    page.send(Message::Binary(vec![4].into())).await?;
    page.close(None).await?;
    assert!(seat.finished.await?.is_ok());
    assert_eq!(seat.output.bytes(), vec![1, 2, 3, 4]);
    Ok(())
}

#[tokio::test]
async fn a_second_page_is_refused_while_one_holds_the_microphone() -> TestResult {
    let seat = start_seat(Duration::from_secs(30)).await?;
    let _first = attached_page(seat.address).await?;
    let mut second = connect_allowed(seat.address, "/audio").await?;
    let refusal = close_of(&next_message(&mut second).await?);
    assert_eq!(
        refusal,
        Some((1013, "Another page holds the microphone".to_owned()))
    );
    Ok(())
}

#[tokio::test]
async fn no_page_within_the_wait_is_a_timeout() -> TestResult {
    let seat = start_seat(Duration::from_millis(200)).await?;
    let failure = seat.finished.await?.err().ok_or("expected a failure")?;
    assert_eq!(failure.message, "the page did not attach");
    Ok(())
}
