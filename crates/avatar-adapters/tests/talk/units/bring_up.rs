use avatar_conversation::talk::ports::{Readiness, Units};

use super::{text, units};
use crate::mock::MockServer;
use crate::tools::{TestResult, Tools};

const RECORDING: &str = "echo \"$@\" >> \"${0%/*}/log\"";
const BROUGHT_UP: &str = "status\n--user start avatar-conversation.target\n--user is-failed avatar-audio avatar-control-server avatar-room-microphone\nstatus\n";
const RUNNING: &str = "case \"$1\" in status) echo '{\"running\": true}';; esac";
const STARTS_ON_DEMAND: &str = "case \"$1\" in status) [ -e \"${0%/*}/up\" ] && echo '{\"running\": true}' || echo '{\"running\": false}';; start) : > \"${0%/*}/up\";; esac";
const FAILING: &str =
    "echo \"$@\" >> \"${0%/*}/log\"; [ \"$2\" = is-failed ] && echo failed; exit 0";
const FAILED_MESSAGE: &str = "An avatar service failed; inspect systemctl --user status avatar-*";
const STARTED_FIRST: &str = "status\nstart\n--user start avatar-window.target\n--user is-failed avatar-control-server avatar-renderer avatar-browser\nstatus\n";
const HEALTHY: &str = r#"{"status": "ok", "rendererConnected": true}"#;

fn recording_tools(audio_body: &str) -> Result<Tools, Box<dyn std::error::Error>> {
    let audio = format!("{RECORDING}; {audio_body}");
    let systemctl = format!("{RECORDING}; exit 0");
    Tools::with(&[("systemctl", &systemctl), ("avatar-audio", &audio)])
}

#[test]
fn bringing_up_starts_the_target_and_waits_for_the_health_checks() -> TestResult {
    let server = MockServer::start(200, HEALTHY)?;
    let tools = recording_tools(RUNNING)?;
    let checks = [Readiness::ControlServer, Readiness::Renderer];
    let outcome = units(&tools, server.port, 1).bring_up("avatar-conversation.target", &checks);
    assert_eq!(text(outcome), "");
    assert_eq!(tools.read("log"), BROUGHT_UP);
    assert_eq!(server.paths(), ["/health", "/health"]);
    Ok(())
}

#[test]
fn audio_that_is_not_running_is_started_first() -> TestResult {
    let tools = recording_tools(STARTS_ON_DEMAND)?;
    let outcome = units(&tools, 1, 1).bring_up("avatar-window.target", &[]);
    assert_eq!(text(outcome), "");
    assert_eq!(tools.read("log"), STARTED_FIRST);
    Ok(())
}

#[test]
fn a_failed_service_stops_the_target_and_reports_the_python_message() -> TestResult {
    let tools = Tools::with(&[("systemctl", FAILING), ("avatar-audio", RUNNING)])?;
    let target = "avatar-window.target";
    let outcome = units(&tools, 1, 1).bring_up(target, &[Readiness::Renderer]);
    assert_eq!(text(outcome), FAILED_MESSAGE);
    assert!(
        tools
            .read("log")
            .ends_with("--user stop avatar-window.target\n")
    );
    Ok(())
}
