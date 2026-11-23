use std::time::Duration;

use serde_json::Value;

use super::programs::Programs;
use crate::process::{Captured, run_captured};

const PANE_LIST_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListedPane {
    pub pane_id: String,
    pub harness_session: Option<String>,
    pub directory: String,
}

pub struct Herdr {
    programs: Programs,
}

impl Herdr {
    #[must_use]
    pub const fn new(programs: Programs) -> Self {
        Self { programs }
    }

    #[must_use]
    pub fn live_panes(&self) -> Vec<ListedPane> {
        let command = self.programs.words(&["herdr", "pane", "list"]);
        run_captured(&command, &[], PANE_LIST_TIMEOUT)
            .ok()
            .filter(Captured::succeeded)
            .map(|captured| parse_panes(&captured.stdout))
            .unwrap_or_default()
    }
}

fn parse_panes(stdout: &[u8]) -> Vec<ListedPane> {
    let Ok(document) = serde_json::from_slice::<Value>(stdout) else {
        return Vec::new();
    };
    let panes = document.pointer("/result/panes").and_then(Value::as_array);
    panes.map_or_else(Vec::new, |panes| panes.iter().filter_map(pane_of).collect())
}

fn pane_of(pane: &Value) -> Option<ListedPane> {
    let session = pane.get("agent_session");
    Some(ListedPane {
        pane_id: pane.get("pane_id")?.as_str()?.to_owned(),
        harness_session: session.and_then(harness_session),
        directory: pane
            .get("cwd")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    })
}

fn harness_session(session: &Value) -> Option<String> {
    let reports_id = session.get("kind").and_then(Value::as_str) == Some("id");
    let harness = session.get("agent").and_then(Value::as_str)?;
    let value = session.get("value").and_then(Value::as_str)?;
    let known = ["claude", "codex", "opencode"].contains(&harness);
    (reports_id && known && !value.is_empty()).then(|| value.to_owned())
}
