use super::ControlLevels;

const URL_WITH_NOTHING_LISTENING: &str = "ws://127.0.0.1:1";

#[test]
fn connecting_outside_any_runtime_reports_an_unreachable_control_server_instead_of_panicking() {
    let connected =
        std::thread::spawn(|| ControlLevels::connect(URL_WITH_NOTHING_LISTENING).is_ok());
    let outcome = connected.join();
    assert!(
        matches!(outcome, Ok(false)),
        "connect panicked or claimed a connection"
    );
}
