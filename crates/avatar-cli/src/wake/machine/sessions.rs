use std::collections::HashMap;

use avatar_adapters::commands::{Herdr, Programs};
use avatar_adapters::process::run_captured;
use avatar_routing::names::session::Session;
use avatar_wake::panes::{LivePane, live_sessions};
use serde_json::Value;
use std::time::Duration;

const LISTING_TIMEOUT: Duration = Duration::from_secs(20);

fn servants_by_pane() -> HashMap<String, String> {
    let command = ["a2a", "list", "--json"].map(str::to_owned);
    let Ok(captured) = run_captured(&command, &[], LISTING_TIMEOUT) else {
        return HashMap::new();
    };
    let listing: Value = serde_json::from_slice(&captured.stdout).unwrap_or(Value::Null);
    let sessions = listing
        .as_object()
        .into_iter()
        .flat_map(|object| object.values());
    sessions.filter_map(servant_of).collect()
}

fn servant_of(session: &Value) -> Option<(String, String)> {
    let pane = session.get("paneId")?.as_str()?;
    let servant = session.get("servant")?.as_str()?;
    Some((pane.to_owned(), servant.to_owned()))
}

pub(crate) fn local_sessions() -> Vec<Session> {
    let servants = servants_by_pane();
    let panes: Vec<LivePane> = Herdr::new(Programs::on_path())
        .live_panes()
        .into_iter()
        .map(|pane| LivePane {
            servant: servants.get(&pane.pane_id).cloned(),
            pane_id: pane.pane_id,
        })
        .collect();
    live_sessions(&panes)
}
