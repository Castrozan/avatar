use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::ports::Readiness;
use avatar_conversation::talk::ports::exec::run_checked;
use avatar_conversation::talk::state::seat::seat_microphone_ready;

use super::SystemdUnits;

pub(super) fn all_ready(
    units: &mut SystemdUnits,
    checks: &[Readiness],
) -> Result<bool, TalkFailure> {
    checks
        .iter()
        .try_fold(true, |ready, check| still_ready(units, ready, *check))
}

fn still_ready(
    units: &mut SystemdUnits,
    ready: bool,
    check: Readiness,
) -> Result<bool, TalkFailure> {
    if !ready {
        return Ok(false);
    }
    is_ready(units, check)
}

fn is_ready(units: &mut SystemdUnits, check: Readiness) -> Result<bool, TalkFailure> {
    match check {
        Readiness::ControlServer => Ok(units.health.control_server_ready()),
        Readiness::Renderer => Ok(units.health.renderer_ready()),
        Readiness::SeatMicrophone(seat) => {
            Ok(seat_microphone_ready(seat, &existing_sources(units)?))
        }
    }
}

fn existing_sources(units: &mut SystemdUnits) -> Result<Vec<String>, TalkFailure> {
    let argv = ["pactl", "list", "short", "sources"];
    let output = run_checked(&mut units.process, &argv, None)?;
    let names = output.stdout.lines().filter_map(source_name);
    Ok(names.map(str::to_owned).collect())
}

fn source_name(line: &str) -> Option<&str> {
    line.split('\t').nth(1)
}
