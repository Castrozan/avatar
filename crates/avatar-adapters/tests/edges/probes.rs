use std::time::Duration;

use avatar_adapters::commands::{Launchctl, SshHost, Systemctl};

use crate::fakes::{TestResult, Tools};

#[test]
fn launchctl_probe_targets_the_gui_domain() -> TestResult {
    let tools = Tools::with(&[("launchctl", "[ \"$1 $2\" = \"print gui/501/label\" ]")])?;
    let probe = Launchctl::new(tools.programs());
    assert!(probe.agent_loaded(501, "label"));
    assert!(!probe.agent_loaded(502, "label"));
    Ok(())
}

#[test]
fn systemctl_probe_reports_the_last_line_of_the_state() -> TestResult {
    let tools = Tools::with(&[("systemctl", "echo noise; echo \"$2-$3\"; exit 3")])?;
    let probe = Systemctl::new(tools.programs());
    assert_eq!(probe.unit_state("hex").as_deref(), Some("is-active-hex"));
    assert!(!probe.unit_active("hex"));
    Ok(())
}

#[test]
fn systemctl_probe_knows_active_units() -> TestResult {
    let tools = Tools::with(&[("systemctl", "echo active")])?;
    assert!(Systemctl::new(tools.programs()).unit_active("hex"));
    Ok(())
}

#[test]
fn ssh_adapter_uses_batch_mode_and_returns_remote_output() -> TestResult {
    let tools = Tools::with(&[("ssh", "echo \"$@\"")])?;
    let host = SshHost::new(tools.programs(), "mac", Duration::from_secs(5));
    let output = host.output_of(&["avatar-wake", "--sessions"]);
    let expected = "-o BatchMode=yes -o ConnectTimeout=5 mac avatar-wake --sessions\n";
    assert_eq!(output.as_deref(), Some(expected));
    Ok(())
}

#[test]
fn ssh_adapter_tells_reachable_from_unreachable() -> TestResult {
    let tools = Tools::with(&[("ssh", "exit 0")])?;
    let host = SshHost::new(tools.programs(), "mac", Duration::from_secs(5));
    assert!(host.reachable());
    tools.write("ssh", "exit 255")?;
    assert!(!host.reachable());
    Ok(())
}
