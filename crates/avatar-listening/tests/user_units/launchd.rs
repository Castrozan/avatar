use std::collections::BTreeMap;
use std::error::Error;

use crate::support::{Outcome, count, fixture, flag, rows, text};
use avatar_listening::user_units::launchd::{LaunchctlPrint, unit_active};
use serde_json::Value;

const FIXTURE: &str = "user_units/fixtures/launchd.json";

#[derive(Default)]
struct ScriptedLaunchctl {
    printed: BTreeMap<String, String>,
    queried_agents: Vec<String>,
    queried_targets: Vec<String>,
}

impl LaunchctlPrint for ScriptedLaunchctl {
    fn printed(&mut self, agent_target: &str) -> String {
        let agent = agent_target.rsplit('/').next().unwrap_or_default();
        self.queried_targets.push(agent_target.to_owned());
        self.queried_agents.push(agent.to_owned());
        self.printed.get(agent).cloned().unwrap_or_default()
    }
}

fn scripted_from(case: &Value) -> Result<ScriptedLaunchctl, Box<dyn Error>> {
    let mut launchctl = ScriptedLaunchctl::default();
    for (agent, output) in case["printed"].as_object().ok_or("printed")? {
        launchctl
            .printed
            .insert(agent.clone(), text(output)?.to_owned());
    }
    Ok(launchctl)
}

fn queried_agents_of(case: &Value) -> Result<Vec<&str>, Box<dyn Error>> {
    rows(&case["queried_agents"])?.iter().map(text).collect()
}

fn check_targets(uid: u32, launchctl: &ScriptedLaunchctl) {
    let prefix = format!("gui/{uid}/");
    assert!(
        launchctl
            .queried_targets
            .iter()
            .all(|target| target.starts_with(&prefix))
    );
}

fn replay(uid: u32, case: &Value) -> Outcome {
    let mut launchctl = scripted_from(case)?;
    let active = unit_active(&mut launchctl, uid, text(&case["unit"])?);
    assert_eq!(active, flag(&case["active"])?, "{case}");
    assert_eq!(launchctl.queried_agents, queried_agents_of(case)?, "{case}");
    check_targets(uid, &launchctl);
    Ok(())
}

#[test]
fn every_oracle_launchd_unit_matches_the_python_agent_probe() -> Outcome {
    let document = fixture(FIXTURE)?;
    let uid = u32::try_from(count(&document["uid"])?)?;
    let cases = rows(&document["cases"])?;
    assert_eq!(cases.len(), 15);
    cases.iter().try_for_each(|case| replay(uid, case))
}
