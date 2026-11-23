use avatar_adapters::commands::avatar_stop;

use crate::tools::{TestResult, Tools};

const UNITS: &str = "avatar-conversation.target avatar-listener.service avatar-remote-seat.service avatar.target avatar-camera.service avatar-browser.service avatar-control-server.service avatar-renderer.service avatar-audio.service";

#[test]
fn it_stops_the_nine_units_then_the_audio_and_keeps_the_systemctl_status() -> TestResult {
    let tools = Tools::with(&[("systemctl", "echo \"$@\" > \"${0%/*}/log\"; exit 3")])?;
    assert_eq!(avatar_stop(&tools.programs(), || 0), 3);
    assert_eq!(tools.read("log"), format!("--user stop {UNITS}\n"));
    Ok(())
}

#[test]
fn a_failing_audio_stop_wins() -> TestResult {
    let tools = Tools::with(&[("systemctl", "exit 3")])?;
    assert_eq!(avatar_stop(&tools.programs(), || 5), 5);
    Ok(())
}

#[test]
fn a_clean_stop_exits_zero() -> TestResult {
    let tools = Tools::with(&[("systemctl", "exit 0")])?;
    assert_eq!(avatar_stop(&tools.programs(), || 0), 0);
    Ok(())
}

#[test]
fn the_audio_stop_runs_even_when_systemctl_is_missing() -> TestResult {
    let tools = Tools::with(&[])?;
    let mut audio_ran = false;
    let status = avatar_stop(&tools.programs(), || {
        audio_ran = true;
        0
    });
    assert_eq!((status, audio_ran), (127, true));
    Ok(())
}
