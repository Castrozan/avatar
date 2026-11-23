use std::error::Error;
use std::sync::Arc;

use avatar_listening::barge_in::reply_link::control_socket::ReplyLink;
use serde_json::{Value, json};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use super::mock_server::{MockCommand, MockControlServer, WAIT};
use crate::support::Outcome;

pub(super) fn speech_activity(speaking: bool) -> MockCommand {
    notice(&json!({"type": "speechActivity", "speaking": speaking}))
}

pub(super) fn notice(value: &Value) -> MockCommand {
    MockCommand::Notice(value.to_string())
}

struct LinkControl {
    link: Arc<ReplyLink>,
    follower: JoinHandle<()>,
    stopping: watch::Sender<bool>,
}

pub(super) struct Session {
    pub(super) server: MockControlServer,
    control: LinkControl,
}

impl Session {
    pub(super) async fn following() -> Result<Self, Box<dyn Error>> {
        let mut server = MockControlServer::start().await?;
        let control = spawn_link(&server.url);
        expect_identification(&mut server).await?;
        Ok(Self { server, control })
    }

    pub(super) fn send_json(&self, value: &Value) {
        self.server.send(notice(value));
    }

    pub(super) async fn stop_link_speaking(&self) {
        self.control.link.stop().await;
    }

    pub(super) async fn expect_speaking(&self, speaking: bool) -> Outcome {
        timeout(
            WAIT,
            poll_until(|| self.control.link.speaking() == speaking),
        )
        .await?;
        Ok(())
    }

    pub(super) fn stop_following(&self) -> Outcome {
        self.control.stopping.send(true)?;
        Ok(())
    }

    pub(super) async fn follower_finished(self) -> Outcome {
        timeout(WAIT, self.control.follower).await??;
        Ok(())
    }
}

fn spawn_link(url: &str) -> LinkControl {
    let (stopping, stop_signal) = watch::channel(false);
    let link = ReplyLink::new(url, stop_signal);
    let follower = tokio::spawn(follow_task(Arc::clone(&link)));
    LinkControl {
        link,
        follower,
        stopping,
    }
}

async fn follow_task(link: Arc<ReplyLink>) {
    link.follow().await;
}

pub(super) async fn expect_message(server: &mut MockControlServer, expected: Value) -> Outcome {
    assert_eq!(server.next_message().await?, expected);
    Ok(())
}

pub(super) async fn expect_identification(server: &mut MockControlServer) -> Outcome {
    expect_message(server, json!({"type": "identify", "role": "agent"})).await?;
    expect_message(server, json!({"type": "followSpeech"})).await
}

async fn poll_until(condition: impl Fn() -> bool) {
    while !condition() {
        tokio::task::yield_now().await;
    }
}
