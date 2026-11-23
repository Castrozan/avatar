use super::machine::{Machine, TestResult, text};

const FAILED_SERVICE: &str = r#"case "$*" in *is-failed*) echo failed;; *) exit 3;; esac"#;

#[test]
fn a_failed_avatar_service_is_reported_and_the_window_stays_hidden() -> TestResult {
    let machine = Machine::with_tools(&[
        ("systemctl", FAILED_SERVICE),
        ("avatar-command", r#"echo "$@" >> "${0%/*}/../log""#),
        ("hyprctl", r#"echo "$@" >> "${0%/*}/../log""#),
    ])?;
    let output = machine.run(&["window-reveal"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("An avatar service failed"));
    assert_eq!(machine.logged(), "");
    Ok(())
}
