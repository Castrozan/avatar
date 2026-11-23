use avatar_listening::barge_in::reply_link::control_socket::ReplyLink;
use serde_json::json;
use tokio::sync::watch;

use crate::support::Outcome;

mod harness;
mod mock_server;

use harness::{Session, expect_identification, expect_message, speech_activity};
use mock_server::MockCommand;

#[tokio::test]
async fn the_link_identifies_then_follows_speech_on_connect() -> Outcome {
    let session = Session::following().await?;
    session.stop_following()
}

#[tokio::test]
async fn speaking_notices_drive_the_state_and_stop_is_sent_on_the_same_connection() -> Outcome {
    let mut session = Session::following().await?;
    session.server.send(speech_activity(true));
    session.expect_speaking(true).await?;
    session.stop_link_speaking().await;
    expect_message(&mut session.server, json!({"type": "stopSpeaking"})).await?;
    session.server.send(speech_activity(false));
    session.expect_speaking(false).await?;
    session.stop_following()
}

#[tokio::test]
async fn other_server_messages_leave_the_speaking_state_alone() -> Outcome {
    let session = Session::following().await?;
    session.server.send(speech_activity(true));
    session.send_json(&json!({"type": "characterChanged", "name": "Voyager"}));
    session.send_json(&json!({"type": "stopSpeakingAck", "status": "stopped"}));
    session.server.send(speech_activity(false));
    session.expect_speaking(false).await?;
    session.stop_following()
}

#[tokio::test]
async fn a_lost_server_clears_speaking_and_the_link_reconnects() -> Outcome {
    let mut session = Session::following().await?;
    session.server.send(speech_activity(true));
    session.expect_speaking(true).await?;
    session.server.send(MockCommand::Close);
    session.expect_speaking(false).await?;
    expect_identification(&mut session.server).await?;
    session.stop_following()
}

#[tokio::test]
async fn a_speech_activity_without_a_flag_drops_the_connection_and_reconnects() -> Outcome {
    let mut session = Session::following().await?;
    session.server.send(speech_activity(true));
    session.expect_speaking(true).await?;
    session.send_json(&json!({"type": "speechActivity"}));
    session.expect_speaking(false).await?;
    expect_identification(&mut session.server).await?;
    session.stop_following()
}

#[tokio::test]
async fn a_link_never_connected_sends_nothing_when_asked_to_stop() -> Outcome {
    let (_stopping, stop_signal) = watch::channel(false);
    let link = ReplyLink::new("ws://127.0.0.1:1", stop_signal);
    link.stop().await;
    assert!(!link.speaking());
    Ok(())
}

#[tokio::test]
async fn setting_stopping_ends_the_follow_loop_once_the_connection_is_gone() -> Outcome {
    let session = Session::following().await?;
    session.stop_following()?;
    session.server.send(MockCommand::Close);
    session.follower_finished().await
}
