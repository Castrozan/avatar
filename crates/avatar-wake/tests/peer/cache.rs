use std::collections::HashMap;
use std::error::Error;

use avatar_routing::names::session::Session;
use avatar_wake::peer::cache::PeerCache;
use avatar_wake::reach::CommandRunner;
use serde_json::{Value, json};

use super::support::{cases, number, replies_of, strings};

const FIXTURE: &str = include_str!("fixtures/peer.json");

struct ScriptedRunner {
    replies: HashMap<String, Option<String>>,
}

impl CommandRunner for ScriptedRunner {
    fn stdout_of(&self, command: &[String], _timeout_seconds: u64) -> Option<String> {
        self.replies.get(&command[5]).cloned().flatten()
    }
}

#[test]
fn the_peer_cache_refreshes_once_per_interval_and_never_overlaps() -> Result<(), Box<dyn Error>> {
    cases(FIXTURE, "cache")?.iter().for_each(check_cache_case);
    Ok(())
}

fn check_cache_case(case: &Value) {
    let runner = ScriptedRunner {
        replies: replies_of(&case["replies"]),
    };
    let actual = run_script(&mut PeerCache::new(strings(&case["hosts"])), case, &runner);
    assert_eq!(Value::Array(actual), case["expected"]);
}

fn run_script(cache: &mut PeerCache, case: &Value, runner: &ScriptedRunner) -> Vec<Value> {
    let mut pending = false;
    case["steps"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|step| run_step(cache, step, runner, &mut pending))
        .collect()
}

fn run_step(
    cache: &mut PeerCache,
    step: &Value,
    runner: &ScriptedRunner,
    pending: &mut bool,
) -> Value {
    if step["op"] == "complete" {
        return complete(cache, runner, pending);
    }
    let started = cache.begin_refresh_if_due(number(&step["now"]));
    *pending |= started;
    observe(cache, step["op"].as_str(), started)
}

fn complete(cache: &mut PeerCache, runner: &ScriptedRunner, pending: &mut bool) -> Value {
    if *pending {
        cache.refresh(runner);
        *pending = false;
    }
    json!({"completed": !*pending})
}

fn observe(cache: &PeerCache, op: Option<&str>, started: bool) -> Value {
    match op {
        Some("known") => json!({"started": started, "sessions": sessions_json(cache)}),
        Some("discord") => json!({"started": started, "discord_host": cache.discord_call_host()}),
        _ => json!({"started": started, "answering": cache.answering_hosts()}),
    }
}

fn sessions_json(cache: &PeerCache) -> Vec<Value> {
    cache.sessions().iter().map(session_json).collect()
}

fn session_json(session: &Session) -> Value {
    json!({"servant": session.servant, "pane": session.pane, "host": session.host})
}
