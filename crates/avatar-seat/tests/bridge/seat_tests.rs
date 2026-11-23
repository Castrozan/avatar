use super::frames::*;
use super::helpers::*;

use super::fakes::VOICE;
use super::harness::{Outcome, eventually, start};
use serde_json::{Value, json};
use tokio::time::timeout;

#[tokio::test]
async fn the_page_and_its_health_are_served_and_nothing_else() -> Outcome<()> {
    let running = start(true, LONG).await?;
    let (page, missing) = (
        running.get("/").await?,
        running.get("/listener.json").await?,
    );
    assert_eq!((page, missing.0), ((200, "<p>seat</p>".to_owned()), 404));
    assert_eq!(flags(&wait_meeting(&running).await?), (json!(false), json!(true)));
    Ok(())
}

#[tokio::test]
async fn audio_travels_both_ways_while_a_machine_holds_the_seat() -> Outcome<()> {
    let running = start(true, LONG).await?;
    let mut socket = attach(&running).await?;
    assert_eq!(collect_voice(&mut socket).await?, VOICE);
    assert_eq!(running.health().await?["attached"], true);
    socket.close(None).await?;
    wait_released(&running).await?;
    assert!(eventually(|| running.heard.microphone_bytes() == MICROPHONE).await);
    Ok(())
}

#[tokio::test]
async fn a_page_from_another_origin_cannot_reach_the_audio_or_the_meeting() -> Outcome<()> {
    let running = start(true, LONG).await?;
    let audio = running.socket("/audio", Some("https://example.com")).await;
    let meeting = running
        .socket("/meeting", Some("https://example.com"))
        .await;
    assert!(audio.is_err_and(|error| error.to_string().contains("403")));
    assert!(meeting.is_err());
    Ok(())
}

#[tokio::test]
async fn a_second_machine_is_refused_while_the_seat_is_held() -> Outcome<()> {
    let running = start(true, LONG).await?;
    let _holder = attach(&running).await?;
    let mut intruder = running.socket("/audio", None).await?;
    assert!(is_taken(&next_frame(&mut intruder).await?));
    Ok(())
}

#[tokio::test]
async fn a_page_failure_shows_in_the_health_and_a_forged_one_does_not() -> Outcome<()> {
    let running = start(true, LONG).await?;
    assert_eq!(running.fetch("/failure?forged", "cross-site").await?.0, 403);
    assert_eq!(running.health().await?["failure"], Value::Null);
    running.get("/failure?NotAllowedError%3A%20denied").await?;
    assert_eq!(
        running.health().await?["failure"],
        "NotAllowedError: denied"
    );
    Ok(())
}

#[tokio::test]
async fn a_seat_whose_audio_tools_cannot_start_is_released() -> Outcome<()> {
    let running = start(false, LONG).await?;
    let mut holder = running.socket("/audio", None).await?;
    timeout(FIVE_SECONDS, drain_frames(&mut holder)).await?;
    assert_eq!(wait_released(&running).await?["attached"], false);
    Ok(())
}

#[tokio::test]
async fn the_bridge_ends_when_no_machine_takes_the_seat() -> Outcome<()> {
    let running = start(true, (50, 50)).await?;
    timeout(FIVE_SECONDS, running.serving).await??;
    Ok(())
}

#[tokio::test]
async fn the_bridge_ends_soon_after_its_machine_leaves() -> Outcome<()> {
    let running = start(true, (30_000, 50)).await?;
    attach(&running).await?.close(None).await?;
    timeout(FIVE_SECONDS, running.serving).await??;
    Ok(())
}
