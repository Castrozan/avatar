const REMOTE_WINDOW_PORT: &str = "8770";
const REMOTE_SEAT_PORT: &str = "8769";

#[must_use]
pub fn page_origins(port: &str) -> [String; 2] {
    [
        format!("http://localhost:{port}"),
        format!("http://127.0.0.1:{port}"),
    ]
}

#[must_use]
pub fn control_origins(renderer_port: &str) -> Vec<String> {
    let mut origins = page_origins(renderer_port).to_vec();
    origins.extend(page_origins(REMOTE_WINDOW_PORT));
    origins.extend(page_origins(REMOTE_SEAT_PORT));
    origins
}
