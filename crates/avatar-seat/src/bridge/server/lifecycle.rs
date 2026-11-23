use std::time::Duration;

use tokio::sync::watch;
use tokio::time::timeout;

use crate::bridge::state::{BridgeLog, SeatPresence};

async fn next_attachment(
    seat: &mut watch::Receiver<SeatPresence>,
    seen: u64,
    grace: Duration,
) -> Option<u64> {
    let waited = timeout(grace, seat.wait_for(|presence| presence.attachments > seen)).await;
    waited.ok()?.ok().map(|presence| presence.attachments)
}

async fn until_detached(seat: &mut watch::Receiver<SeatPresence>) -> bool {
    seat.wait_for(|presence| !presence.attached).await.is_ok()
}

pub async fn stay_while_taken(
    mut seat: watch::Receiver<SeatPresence>,
    graces: (Duration, Duration),
    log: &dyn BridgeLog,
) {
    let (mut grace, mut seen) = (graces.0, 0);
    while let Some(attachments) = next_attachment(&mut seat, seen, grace).await
        && until_detached(&mut seat).await
    {
        (grace, seen) = (graces.1, attachments);
    }
    log.log("No machine holds the remote seat");
}
