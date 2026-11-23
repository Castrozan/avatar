use std::sync::Mutex;
use std::time::Duration;

use avatar_seat::host::state::{HostProbe, HostSnapshot, HostState, ProbeFailure};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Scenario {
    reads: Vec<Value>,
    snapshots: Vec<Snapshot>,
}

#[derive(Deserialize, PartialEq, Debug)]
struct Snapshot {
    listening: bool,
    meeting: bool,
}

struct Scripted(Mutex<Vec<Value>>);

impl HostProbe for Scripted {
    fn read(&self) -> Result<HostSnapshot, ProbeFailure> {
        let next = self.0.lock().map_err(|_| ProbeFailure)?.remove(0);
        let state: Snapshot = serde_json::from_value(next).map_err(|_| ProbeFailure)?;
        Ok(HostSnapshot {
            listening: state.listening,
            meeting: state.meeting,
        })
    }
}

fn shown(snapshot: HostSnapshot) -> Snapshot {
    Snapshot {
        listening: snapshot.listening,
        meeting: snapshot.meeting,
    }
}

async fn replay(scenario: &Scenario) -> Vec<Snapshot> {
    let state = HostState::new(Scripted(Mutex::new(scenario.reads.clone())));
    let mut seen = vec![shown(state.snapshot())];
    for _ in &scenario.reads {
        state.refresh().await;
        seen.push(shown(state.snapshot()));
    }
    seen
}

async fn check_all(scenarios: &[Scenario]) {
    for scenario in scenarios {
        assert_eq!(replay(scenario).await, scenario.snapshots);
    }
}

#[tokio::test]
async fn every_refresh_scenario_keeps_the_last_good_snapshot() {
    let scenarios: Vec<Scenario> =
        serde_json::from_str(include_str!("fixtures/refresh.json")).unwrap_or_default();
    assert!(scenarios.len() >= 5);
    check_all(&scenarios).await;
}

#[tokio::test(start_paused = true)]
async fn following_refreshes_every_two_seconds() {
    let state = HostState::new(Scripted(Mutex::new(vec![
        json!({"listening": true, "meeting": false});
        3
    ])));
    let followed = tokio::time::timeout(Duration::from_secs(3), state.follow()).await;
    assert!(followed.is_err());
    assert!(state.snapshot().listening);
}
