use std::time::Duration;

use avatar_live::boundaries::Console;
use avatar_live::session::MODEL;
use avatar_transcript::json::encode;
use tokio::time::timeout;

use super::launch::Running;
use crate::daemons::live::records::StandardOutput;

const CLOSE_AND_DRAIN: Duration = Duration::from_secs(15);

async fn close_voice(running: &mut Running) {
    running.voice.close();
    if timeout(CLOSE_AND_DRAIN, &mut running.actor).await.is_err() {
        running.actor.abort();
    }
}

pub(super) async fn finish(mut running: Running) {
    running.microphone.abort();
    running.follower.abort();
    close_voice(&mut running).await;
    if let Ok(mut conversation) = running.hall.await {
        conversation.finish();
        StandardOutput.print(&encode(&conversation.summary(MODEL)));
    }
}
