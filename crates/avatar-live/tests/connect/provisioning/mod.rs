mod access;
mod containment;
mod ledger;
mod server;

use std::sync::{Arc, Mutex, PoisonError};

use avatar_live::client::provisioning::{AGENT_NAME, AgentApi, agent_settings, provision};
use serde_json::{Value, json};

use self::ledger::{KEY, Ledger};
use self::server::{Shared, start};
use crate::support::{Outcome, block_on};

type Failure = Box<dyn std::error::Error>;

async fn provisioned(ledger: &Shared, key: &str) -> Result<String, Failure> {
    let address = start(Arc::clone(ledger)).await?;
    let api = AgentApi::new(&format!("http://{address}/"), key);
    Ok(provision(&api).await?)
}

fn inspect<Output>(ledger: &Shared, look: impl FnOnce(&Ledger) -> Output) -> Output {
    look(&ledger.lock().unwrap_or_else(PoisonError::into_inner))
}

fn fresh() -> Shared {
    Arc::new(Mutex::new(Ledger::default()))
}

#[test]
fn the_first_start_creates_the_tool_and_an_agent_that_uses_it() -> Outcome {
    let ledger = fresh();
    let agent = block_on(provisioned(&ledger, KEY))??;
    assert_eq!(agent, "agents-1");
    let tool = inspect(&ledger, |found| found.tools[0].clone());
    assert_eq!(tool["tool_config"]["name"], "hand_off_to_agent");
    let created = inspect(&ledger, |found| found.agents[0].clone());
    assert_eq!(created, merged_with_id(agent_settings("tools-1")));
    let writes = inspect(&ledger, |found| found.writes.clone());
    assert_eq!(writes, ["create tools", "create agents"]);
    Ok(())
}

fn merged_with_id(mut settings: Value) -> Value {
    settings["agent_id"] = json!("agents-1");
    settings
}

#[test]
fn a_later_start_finds_both_by_name_and_writes_nothing() -> Outcome {
    let ledger = fresh();
    let first = block_on(provisioned(&ledger, KEY))??;
    let second = block_on(provisioned(&ledger, KEY))??;
    assert_eq!(first, second);
    assert_eq!(inspect(&ledger, |found| found.writes.len()), 2);
    Ok(())
}

fn drift(ledger: &Shared, change: impl FnOnce(&mut Value)) {
    let mut held = ledger.lock().unwrap_or_else(PoisonError::into_inner);
    change(&mut held.agents[0]);
}

fn add_server_metadata(agent: &mut Value) {
    agent["metadata"] = json!({"created_at_unix_secs": 1});
}

fn change_voice(agent: &mut Value) {
    agent["conversation_config"]["tts"]["voice_id"] = json!("other");
}

#[test]
fn server_side_extras_are_kept_without_a_write() -> Outcome {
    let ledger = fresh();
    block_on(provisioned(&ledger, KEY))??;
    drift(&ledger, add_server_metadata);
    block_on(provisioned(&ledger, KEY))??;
    assert_eq!(inspect(&ledger, |found| found.writes.len()), 2);
    Ok(())
}

#[test]
fn a_drifted_setting_is_patched_back() -> Outcome {
    let ledger = fresh();
    block_on(provisioned(&ledger, KEY))??;
    drift(&ledger, change_voice);
    block_on(provisioned(&ledger, KEY))??;
    let agent = inspect(&ledger, |found| found.agents[0].clone());
    let wanted = agent_settings("tools-1");
    assert_eq!(agent["conversation_config"], wanted["conversation_config"]);
    Ok(())
}

#[test]
fn a_similar_name_is_not_taken_for_the_agent() -> Outcome {
    let ledger = fresh();
    let decoy = json!({"agent_id": "decoy", "name": format!("{AGENT_NAME}-old")});
    ledger
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .agents
        .push(decoy);
    assert_eq!(block_on(provisioned(&ledger, KEY))??, "agents-2");
    Ok(())
}
