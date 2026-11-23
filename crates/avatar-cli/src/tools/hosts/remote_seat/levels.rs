use std::process::Stdio;

use avatar_seat::host::levels::{LevelSource, Sink, SinkReadFailure};
use tokio::process::Command;

pub(super) struct PactlLevels;

async fn listing() -> Option<Vec<u8>> {
    let output = Command::new("pactl")
        .args(["--format=json", "list", "sinks"])
        .stdin(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await
        .ok()?;
    output.status.success().then_some(output.stdout)
}

impl LevelSource for PactlLevels {
    async fn read_sinks(&self) -> Result<Vec<Sink>, SinkReadFailure> {
        let listed = listing().await.ok_or(SinkReadFailure)?;
        serde_json::from_slice(&listed).map_err(|_| SinkReadFailure)
    }
}
