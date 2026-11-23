use std::error::Error;
use std::path::Path;

use avatar_doctor::switch::moment::{Boundary, Switch, switches_in};
use avatar_doctor::switch::record::logged_records;
use avatar_doctor::switch::report::stage::took;

fn recorded_switches() -> Result<Vec<Switch>, Box<dyn Error>> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/switch/fixtures/recorded-wake.log");
    let text = std::fs::read_to_string(path)?;
    Ok(switches_in(&logged_records(text.lines())?)?)
}

fn decision_seconds(switch: &Switch) -> Result<f64, Box<dyn Error>> {
    took(
        &switch.moments,
        Boundary::DecisionStarted,
        Boundary::DecisionDone,
    )
    .ok_or_else(|| "no decision".into())
}

#[test]
fn todays_recorded_switches_time_the_routing_decision() -> Result<(), Box<dyn Error>> {
    let switches = recorded_switches()?;
    let [tamamo, anastasia] = switches.as_slice() else {
        return Err("expected two switches".into());
    };
    assert_eq!(identity(tamamo), (Some("Tamamo"), Some(75), None));
    assert_eq!(identity(anastasia), (Some("Anastasia"), Some(80), None));
    assert!((decision_seconds(tamamo)? - 0.628).abs() < 1e-6);
    assert!((decision_seconds(anastasia)? - 0.380).abs() < 1e-6);
    Ok(())
}

fn identity(switch: &Switch) -> (Option<&str>, Option<i64>, Option<&str>) {
    (
        switch.servant.as_deref(),
        switch.sequence,
        switch.delivered.as_deref(),
    )
}

#[test]
fn the_assembly_between_wake_and_decision_is_measured() -> Result<(), Box<dyn Error>> {
    let switches = recorded_switches()?;
    let assembly = took(
        &switches[0].moments,
        Boundary::WakeRead,
        Boundary::DecisionStarted,
    );
    assert!((assembly.ok_or("no assembly")? - 0.036_468).abs() < 1e-6);
    Ok(())
}

#[test]
fn a_switch_without_a_decision_keeps_only_the_recorded_boundaries() -> Result<(), Box<dyn Error>> {
    let switches = recorded_switches()?;
    let unrecorded = Boundary::ALL
        .into_iter()
        .filter(|boundary| switches[0].moments.get(*boundary).is_none())
        .count();
    assert_eq!(unrecorded, Boundary::ALL.len() - 3);
    Ok(())
}

#[test]
fn a_broken_line_and_a_non_switch_line_measure_nothing() -> Result<(), Box<dyn Error>> {
    let lines = ["{broken", "wake Voyager at w0:p7"];
    assert!(switches_in(&logged_records(lines)?)?.is_empty());
    Ok(())
}
