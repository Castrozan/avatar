use avatar_adapters::commands::{A2aLookup, TalkProcess};
use avatar_conversation::talk::ports::AgentPane;

use crate::failure::message_of;
use crate::tools::{TestResult, Tools};

const LISTING: &str =
    r#"echo '{"a": {"paneId": "p1", "servant": "Ada"}, "b": {"paneId": "p2", "servant": "Bo"}}'"#;
const REMOTE_ARGUMENTS: &str = "-o\nBatchMode=yes\n-o\nConnectTimeout=10\nmac\na2a list --json\n";

fn pane(host: Option<&str>, name: &str) -> AgentPane {
    AgentPane {
        host: host.map(str::to_owned),
        pane: Some(name.to_owned()),
    }
}

fn lookup(tools: &Tools) -> A2aLookup {
    A2aLookup::new(TalkProcess::new(tools.programs()))
}

#[test]
fn the_servant_of_a_local_pane_is_found_in_the_listing() -> TestResult {
    let tools = Tools::with(&[("a2a", LISTING)])?;
    let found = lookup(&tools).servant_at(&pane(None, "p2"));
    assert_eq!(found, Ok(Some("Bo".to_owned())));
    assert_eq!(lookup(&tools).servant_at(&pane(None, "p9")), Ok(None));
    Ok(())
}

#[test]
fn a_remote_pane_is_reached_over_batch_mode_ssh_with_a_quoted_command() -> TestResult {
    let script = format!("printf '%s\\n' \"$@\" > \"$0.log\"; {LISTING}");
    let tools = Tools::with(&[("ssh", &script)])?;
    let found = lookup(&tools).servant_at(&pane(Some("mac"), "p1"));
    assert_eq!(found, Ok(Some("Ada".to_owned())));
    assert_eq!(tools.read("ssh.log"), REMOTE_ARGUMENTS);
    Ok(())
}

#[test]
fn an_unparseable_listing_is_a_failure() -> TestResult {
    let tools = Tools::with(&[("a2a", "echo nope")])?;
    assert!(lookup(&tools).servant_at(&pane(None, "p1")).is_err());
    Ok(())
}

#[test]
fn a_failing_listing_reports_the_python_exit_message() -> TestResult {
    let tools = Tools::with(&[("a2a", "exit 2")])?;
    let message = message_of(lookup(&tools).servant_at(&pane(None, "p1")).err());
    assert!(
        message.ends_with("returned non-zero exit status 2."),
        "{message}"
    );
    Ok(())
}
