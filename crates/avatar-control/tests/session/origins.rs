use avatar_control::session::control_origins;

use crate::{TestResult, ensure};

const EXPECTED_ORIGINS: [&str; 6] = [
    "http://127.0.0.1:3000",
    "http://127.0.0.1:8769",
    "http://127.0.0.1:8770",
    "http://localhost:3000",
    "http://localhost:8769",
    "http://localhost:8770",
];

#[test]
fn control_socket_admits_the_renderer_and_the_remote_pages_only() -> TestResult {
    let mut origins = control_origins("3000");
    origins.sort();
    ensure(origins == EXPECTED_ORIGINS, || {
        format!("origins were {origins:?}")
    })
}
