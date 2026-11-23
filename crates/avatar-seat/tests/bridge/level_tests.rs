use super::frames::*;
use super::helpers::*;
use std::time::Duration;

use super::harness::{Outcome, eventually, start};
use futures_util::SinkExt;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn the_levels_arrive_on_attach_and_again_each_time_the_endpoint_is_asked() -> Outcome<()> {
    let running = start(true, LONG).await?;
    quiet_levels(&running);
    let mut socket = attach(&running).await?;
    assert_eq!(next_text(&mut socket).await?, quiet_answer());
    loud_levels(&running);
    assert_eq!(running.get("/levels").await?, (200, String::new()));
    assert_eq!(next_text(&mut socket).await?, loud_answer());
    Ok(())
}

#[tokio::test]
async fn asking_for_levels_with_no_machine_attached_reads_nothing() -> Outcome<()> {
    let running = start(true, LONG).await?;
    assert_eq!(running.get("/levels").await?, (200, String::new()));
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(reads(&running), 0);
    Ok(())
}

#[tokio::test]
async fn another_site_cannot_drive_the_levels() -> Outcome<()> {
    let running = start(true, LONG).await?;
    let mut socket = attach(&running).await?;
    next_text(&mut socket).await?;
    assert_eq!(running.fetch("/levels", "cross-site").await?.0, 403);
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(reads(&running), 1);
    Ok(())
}

#[tokio::test]
async fn the_meeting_page_is_carried_once_at_a_time() -> Outcome<()> {
    let running = start(true, LONG).await?;
    let mut first = running.socket("/meeting", None).await?;
    first
        .send(Message::Binary(MICROPHONE.to_vec().into()))
        .await?;
    let mut second = running.socket("/meeting", None).await?;
    assert!(is_taken(&next_frame(&mut second).await?));
    assert!(eventually(|| running.heard.meeting_bytes() == MICROPHONE).await);
    Ok(())
}

#[tokio::test]
async fn stopping_ends_the_serving_and_closes_the_held_seat() -> Outcome<()> {
    let mut running = start(true, LONG).await?;
    let mut holder = attach(&running).await?;
    running.stop();
    timeout(FIVE_SECONDS, running.serving).await??;
    timeout(FIVE_SECONDS, drain_frames(&mut holder)).await?;
    Ok(())
}
