use avatar_adapters::commands::{
    Endpoints, HealthProbe, ReadinessTiming, SystemdUnits, TalkProcess,
};
use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::ports::{Readiness, Units};
use avatar_floor::quiet::Seat;

use crate::failure::message_of;
use crate::mock::{MockServer, closed_port};
use crate::tools::{TestResult, Tools};

mod bring_up;

const QUICK: ReadinessTiming = ReadinessTiming {
    timeout_seconds: 1.0,
    poll_seconds: 0.05,
};

pub(super) fn units(tools: &Tools, control_port: u16, seat_port: u16) -> SystemdUnits {
    let health = HealthProbe::new(Endpoints {
        control_port,
        seat_port,
    });
    SystemdUnits::new(TalkProcess::new(tools.programs()), health, QUICK)
}

pub(super) fn text(outcome: Result<(), TalkFailure>) -> String {
    message_of(outcome.err())
}

#[test]
fn a_unit_is_active_only_when_systemctl_says_zero() -> TestResult {
    let tools = Tools::with(&[("systemctl", "[ \"$4\" = on ] || exit 3")])?;
    assert!(units(&tools, 1, 1).unit_active("on"));
    assert!(!units(&tools, 1, 1).unit_active("off"));
    Ok(())
}

#[test]
fn waiting_past_the_deadline_names_the_units_and_the_seconds() -> TestResult {
    let tools = Tools::with(&[("systemctl", "exit 0")])?;
    let outcome =
        units(&tools, closed_port()?, 1).wait_until_ready(&["one", "two"], &[Readiness::Renderer]);
    assert_eq!(
        text(outcome),
        "one, two did not become ready within 1 seconds"
    );
    Ok(())
}

#[test]
fn the_remote_seat_listens_only_when_its_health_answers_ok() -> TestResult {
    let (healthy, broken) = (MockServer::start(200, "{}")?, MockServer::start(503, "{}")?);
    let tools = Tools::with(&[])?;
    assert!(units(&tools, 1, healthy.port).remote_seat_listening());
    assert!(!units(&tools, 1, broken.port).remote_seat_listening());
    assert!(!units(&tools, 1, closed_port()?).remote_seat_listening());
    Ok(())
}

#[test]
fn a_seat_microphone_is_ready_when_pactl_lists_its_source() -> TestResult {
    let pactl = "printf '1\\tother\\tx\\n2\\tAvatarRoomMicrophone\\tx\\n'";
    let tools = Tools::with(&[("pactl", pactl), ("systemctl", "exit 0")])?;
    let check = [Readiness::SeatMicrophone(Seat::Room)];
    assert_eq!(
        text(units(&tools, 1, 1).wait_until_ready(&["u"], &check)),
        ""
    );
    Ok(())
}

#[test]
fn a_seat_microphone_that_pactl_does_not_list_never_becomes_ready() -> TestResult {
    let tools = Tools::with(&[
        ("pactl", "printf '1\\tother\\tx\\n'"),
        ("systemctl", "exit 0"),
    ])?;
    let check = [Readiness::SeatMicrophone(Seat::Remote)];
    assert!(
        text(units(&tools, 1, 1).wait_until_ready(&["u"], &check)).contains("did not become ready")
    );
    Ok(())
}
