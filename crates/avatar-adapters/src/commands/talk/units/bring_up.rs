use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::ports::Readiness;
use avatar_conversation::talk::ports::exec::{run_checked, run_unchecked};
use serde_json::Value;

use super::SystemdUnits;
use super::waiting::wait_for_units;

const AUDIO_REPAIR_SECONDS: f64 = 60.0;
const AUDIO_PROBE_SECONDS: f64 = 10.0;
const STOP_SECONDS: f64 = 30.0;
const AUDIO_NOT_READY: &str = "The avatar audio graph did not become ready";
const CONVERSATION_UNITS: [&str; 3] = [
    "avatar-audio",
    "avatar-control-server",
    "avatar-room-microphone",
];
const MEETING_UNITS: [&str; 5] = [
    "avatar-audio",
    "avatar-control-server",
    "avatar-renderer",
    "avatar-browser",
    "avatar-camera",
];
const WINDOW_UNITS: [&str; 3] = ["avatar-control-server", "avatar-renderer", "avatar-browser"];

fn target_units(target: &str) -> Result<&'static [&'static str], TalkFailure> {
    match target {
        "avatar.target" => Ok(&MEETING_UNITS),
        "avatar-conversation.target" => Ok(&CONVERSATION_UNITS),
        "avatar-window.target" => Ok(&WINDOW_UNITS),
        other => Err(TalkFailure::new(format!("'{other}'"))),
    }
}

pub(super) fn bring_up(
    units: &mut SystemdUnits,
    target: &str,
    checks: &[Readiness],
) -> Result<(), TalkFailure> {
    let outcome = start_target(units, target, checks);
    if outcome.is_err() {
        let argv = ["systemctl", "--user", "stop", target];
        let _ = run_unchecked(&mut units.process, &argv, Some(STOP_SECONDS));
    }
    outcome
}

fn start_target(
    units: &mut SystemdUnits,
    target: &str,
    checks: &[Readiness],
) -> Result<(), TalkFailure> {
    start_audio_unless_ready(units)?;
    let argv = ["systemctl", "--user", "start", target];
    run_checked(&mut units.process, &argv, Some(AUDIO_REPAIR_SECONDS))?;
    wait_for_units(units, target_units(target)?, checks)?;
    confirm_audio(units)
}

fn start_audio_unless_ready(units: &mut SystemdUnits) -> Result<(), TalkFailure> {
    if audio_ready(units)? {
        return Ok(());
    }
    let argv = ["avatar-audio", "start"];
    run_checked(&mut units.process, &argv, Some(AUDIO_REPAIR_SECONDS)).map(|_| ())
}

fn confirm_audio(units: &mut SystemdUnits) -> Result<(), TalkFailure> {
    if audio_ready(units)? {
        return Ok(());
    }
    Err(TalkFailure::new(AUDIO_NOT_READY))
}

fn audio_ready(units: &mut SystemdUnits) -> Result<bool, TalkFailure> {
    let argv = ["avatar-audio", "status"];
    let output = run_unchecked(&mut units.process, &argv, Some(AUDIO_PROBE_SECONDS))?;
    if output.status != 0 {
        return Ok(false);
    }
    let status: Value = serde_json::from_str(&output.stdout)
        .map_err(|error| TalkFailure::new(error.to_string()))?;
    Ok(status.get("running") == Some(&Value::Bool(true)))
}
