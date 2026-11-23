use avatar_adapters::commands::{Herdr, ListedPane};

use crate::fakes::{TestResult, Tools};

const LISTING: &str = r#"{"result":{"panes":[
{"pane_id":"w1-1","cwd":"/work","agent_session":{"kind":"id","agent":"claude","value":"abc"}},
{"pane_id":"w1-2","cwd":"/work","agent_session":{"kind":"name","agent":"claude","value":"x"}},
{"pane_id":"w1-3"},
"stray"]}}"#;

fn pane(id: &str, directory: &str, harness: Option<&str>) -> ListedPane {
    ListedPane {
        pane_id: id.to_owned(),
        directory: directory.to_owned(),
        harness_session: harness.map(str::to_owned),
    }
}

#[test]
fn herdr_lists_panes_with_reported_harness_sessions() -> TestResult {
    let body = format!("[ \"$2\" = list ] && cat <<'JSON'\n{LISTING}\nJSON");
    let tools = Tools::with(&[("herdr", &body)])?;
    let expected = vec![
        pane("w1-1", "/work", Some("abc")),
        pane("w1-2", "/work", None),
        pane("w1-3", "", None),
    ];
    assert_eq!(Herdr::new(tools.programs()).live_panes(), expected);
    Ok(())
}

#[test]
fn herdr_failures_and_garbage_yield_no_panes() -> TestResult {
    let tools = Tools::with(&[])?;
    let herdr = Herdr::new(tools.programs());
    assert!(herdr.live_panes().is_empty());
    tools.write("herdr", "echo not-json")?;
    assert!(herdr.live_panes().is_empty());
    Ok(())
}

#[test]
fn a_failing_herdr_yields_no_panes_even_with_output() -> TestResult {
    let body = format!("cat <<'JSON'\n{LISTING}\nJSON\nexit 1");
    let tools = Tools::with(&[("herdr", &body)])?;
    assert!(Herdr::new(tools.programs()).live_panes().is_empty());
    Ok(())
}
