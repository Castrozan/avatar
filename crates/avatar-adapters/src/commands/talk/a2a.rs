use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::ports::AgentPane;
use avatar_conversation::talk::ports::exec::run_checked;
use serde_json::Value;

use super::process::TalkProcess;
use crate::shell_quote::join;

const LISTING_SECONDS: f64 = 20.0;
const REACH_HOST: [&str; 5] = ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=10"];
const NOT_A_LISTING: &str = "a2a list --json did not answer a JSON object";

pub struct A2aLookup {
    process: TalkProcess,
}

impl A2aLookup {
    #[must_use]
    pub const fn new(process: TalkProcess) -> Self {
        Self { process }
    }

    pub fn servant_at(&mut self, agent: &AgentPane) -> Result<Option<String>, TalkFailure> {
        let argv = on_host(agent.host.as_deref(), &["a2a", "list", "--json"]);
        let output = run_checked(&mut self.process, &argv, Some(LISTING_SECONDS))?;
        servant_in(&output.stdout, agent.pane.as_deref())
    }
}

fn servant_in(listing_text: &str, pane: Option<&str>) -> Result<Option<String>, TalkFailure> {
    let listing: Value =
        serde_json::from_str(listing_text).map_err(|error| TalkFailure::new(error.to_string()))?;
    let sessions = listing
        .as_object()
        .ok_or_else(|| TalkFailure::new(NOT_A_LISTING))?;
    let matching = sessions
        .values()
        .find(|session| session_pane(session) == pane);
    Ok(matching.and_then(|session| session.get("servant")?.as_str().map(str::to_owned)))
}

fn session_pane(session: &Value) -> Option<&str> {
    session.get("paneId")?.as_str()
}

fn on_host(host: Option<&str>, command: &[&str]) -> Vec<String> {
    let words: Vec<String> = command.iter().map(|word| (*word).to_owned()).collect();
    let Some(name) = host else {
        return words;
    };
    let mut reach: Vec<String> = REACH_HOST.map(str::to_owned).to_vec();
    reach.extend([name.to_owned(), join(&words)]);
    reach
}
