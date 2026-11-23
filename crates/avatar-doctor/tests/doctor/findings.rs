use avatar_doctor::doctor::{Finding, rendered_json, rendered_text, seat_report};

use super::fake::{FakeProbe, FakeRunner, Reply, answer, answered};

const HOST_REPORT: &str = r#"[{"name": "hex.service", "healthy": true, "detail": "active"}]"#;
const HUB: &str = "hub";
const USER: u32 = 501;

type Answers = Vec<(String, Reply)>;

fn seat_of(answers: Answers, serves: bool) -> Vec<Finding> {
    seat_report(&FakeRunner::new(answers), &FakeProbe { serves }, HUB, USER)
}

fn seat_with(extra: Answers, serves: bool) -> Vec<Finding> {
    let mut answers = vec![answer("avatar-doctor", answered(0, HOST_REPORT, ""))];
    answers.extend(extra);
    seat_of(answers, serves)
}

fn talk(status: &str) -> Answers {
    vec![answer("avatar-talk", answered(0, status, ""))]
}

#[test]
fn a_seat_relays_what_its_host_reports_under_the_host_name() {
    let findings = seat_with(vec![], true);
    assert!(findings.contains(&Finding::new("hub: hex.service", true, "active")));
}

#[test]
fn an_unreachable_host_is_the_finding_and_nothing_is_asked_of_it() {
    let runner = FakeRunner::new(vec![answer(
        "hub true",
        answered(255, "", "ssh: no route\n"),
    )]);
    let findings = seat_report(&runner, &FakeProbe { serves: true }, HUB, USER);
    assert_eq!(
        findings[0],
        Finding::new("reach hub", false, "ssh: no route")
    );
    assert!(!runner.was_asked("avatar-doctor"));
}

#[test]
fn unloaded_mac_agents_are_reported() {
    let findings = seat_with(vec![answer("launchctl", answered(113, "", ""))], true);
    assert!(findings.contains(&Finding::new("wake agent", false, "not loaded")));
    assert!(findings.contains(&Finding::new("conversation agent", false, "not loaded")));
}

#[test]
fn a_silent_wake_page_is_healthy_only_while_a_conversation_listens() {
    let listening = seat_with(talk(r#"{"open": true}"#), false);
    let silent = seat_with(talk(r#"{"open": false}"#), false);
    assert!(listening.contains(&wake(true, "paused while a conversation listens")));
    assert!(silent.contains(&wake(false, "no capture page and no conversation")));
}

fn wake(healthy: bool, detail: &str) -> Finding {
    Finding::new("wake microphone", healthy, detail)
}

#[test]
fn a_command_that_never_returns_becomes_a_finding_not_a_crash() {
    let stuck = Reply::Raised {
        raise: "timeout".to_owned(),
    };
    let findings = seat_of(vec![answer("hub true", stuck)], true);
    assert_eq!(findings.len(), 1);
    assert_eq!(
        (findings[0].name.as_str(), findings[0].healthy),
        ("doctor", false)
    );
}

#[test]
fn text_output_marks_each_finding_and_json_round_trips() -> Result<(), Box<dyn std::error::Error>> {
    let findings = [
        Finding::new("a", true, "fine"),
        Finding::new("b", false, "broken"),
    ];
    assert_eq!(rendered_text(&findings), "ok   a: fine\nFAIL b: broken");
    let parsed: serde_json::Value = serde_json::from_str(&rendered_json(&findings)?)?;
    let broken = serde_json::json!({"name": "b", "healthy": false, "detail": "broken"});
    assert_eq!(parsed[1], broken);
    Ok(())
}

#[test]
fn a_seat_checks_its_own_transcription_even_when_the_host_is_gone() {
    let refusal = answered(1, "", "ConnectionError: Hex refused\n");
    let answers = vec![
        answer("hub true", answered(255, "", "no route\n")),
        answer("avatar-transcribe", refusal),
    ];
    let findings = seat_of(answers, true);
    let transcription = Finding::new("transcription", false, "ConnectionError: Hex refused");
    assert_eq!(findings.last(), Some(&transcription));
}
