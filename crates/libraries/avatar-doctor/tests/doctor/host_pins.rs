use avatar_doctor::doctor::{Finding, all_healthy, host_report};

use super::fake::{FakeRunner, Reply, answer, answered};

pub(super) fn healthy_host() -> Vec<(String, Reply)> {
    let active = answered(0, "active\n", "");
    vec![
        answer("is-active", active),
        answer("avatar-transcribe", answered(0, "", "")),
    ]
}

#[test]
fn a_host_with_everything_running_is_healthy() {
    let findings = host_report(&FakeRunner::new(healthy_host()));
    assert_eq!(findings.len(), 3);
    assert!(all_healthy(&findings));
}

#[test]
fn a_stopped_unit_is_named_with_its_state() {
    let mut answers = vec![answer("hex.service", answered(3, "inactive\n", ""))];
    answers.extend(healthy_host());
    let findings = host_report(&FakeRunner::new(answers));
    assert_eq!(findings[0], Finding::new("hex.service", false, "inactive"));
}

#[test]
fn a_transcriber_that_refuses_reports_its_last_error_line() {
    let refusal = "Traceback\nConnectionError: Hex refused\n";
    let answers = vec![
        answer("is-active", answered(0, "active\n", "")),
        answer("avatar-transcribe", answered(1, "", refusal)),
    ];
    let findings = host_report(&FakeRunner::new(answers));
    assert_eq!(
        findings[2],
        Finding::new("transcription", false, "ConnectionError: Hex refused")
    );
}
