mod mock_server;

use std::fs;
use std::process::Output;

use crate::machine::{Machine, TestResult, text};
use mock_server::MockServer;

const BODY: &[u8] = b"mock model bytes";
const BODY_INTEGRITY: &str = "sha256-USu1Y9SIYRj7n5OUqHpIPjDOm8CXU6MdPebFJALfLdE=";

fn manifest_for(machine: &Machine, port: u16, integrity: &str) -> Result<String, std::io::Error> {
    let path = machine.path("manifest.json");
    let entry = format!(
        r#"{{"downloads":[{{"name":"sample","url":"http://127.0.0.1:{port}/sample.onnx","integrity":"{integrity}","destination":"models/sample.onnx"}}]}}"#
    );
    fs::write(&path, entry)?;
    Ok(path.display().to_string())
}

fn setup(machine: &Machine, manifest: &str) -> Result<Output, Box<dyn std::error::Error>> {
    let data = machine.path("data");
    let output = machine
        .command(&["setup", "--manifest", manifest])
        .env("AVATAR_DATA_DIRECTORY", data)
        .output()?;
    Ok(output)
}

#[test]
fn setup_downloads_into_the_data_directory_once() -> TestResult {
    let machine = Machine::with_tools(&[])?;
    let server = MockServer::serving(BODY)?;
    let manifest = manifest_for(&machine, server.port, BODY_INTEGRITY)?;
    let first = setup(&machine, &manifest)?;
    assert!(first.status.success(), "{}", text(&first.stderr));
    assert_eq!(fs::read(machine.path("data/models/sample.onnx"))?, BODY);
    assert!(text(&first.stdout).contains("sample: installed"));
    Ok(())
}

#[test]
fn setup_does_not_download_what_is_already_verified() -> TestResult {
    let machine = Machine::with_tools(&[])?;
    let server = MockServer::serving(BODY)?;
    let manifest = manifest_for(&machine, server.port, BODY_INTEGRITY)?;
    setup(&machine, &manifest)?;
    let second = setup(&machine, &manifest)?;
    assert!(text(&second.stdout).contains("sample: already present"));
    assert_eq!(server.requests(), 1);
    Ok(())
}

#[test]
fn setup_refuses_content_that_does_not_match_its_digest() -> TestResult {
    let machine = Machine::with_tools(&[])?;
    let server = MockServer::serving(b"something else")?;
    let manifest = manifest_for(&machine, server.port, BODY_INTEGRITY)?;
    let output = setup(&machine, &manifest)?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("failed verification"));
    assert!(!machine.path("data/models/sample.onnx").exists());
    Ok(())
}

#[test]
fn setup_reports_an_unreachable_server() -> TestResult {
    let machine = Machine::with_tools(&[])?;
    let closed = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?
        .local_addr()?
        .port();
    let manifest = manifest_for(&machine, closed, BODY_INTEGRITY)?;
    let output = setup(&machine, &manifest)?;
    assert_eq!(output.status.code(), Some(1));
    assert!(!machine.path("data/models/sample.onnx").exists());
    Ok(())
}
