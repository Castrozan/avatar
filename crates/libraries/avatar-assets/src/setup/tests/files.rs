use std::fs;

use super::support::{FakeTransport, TestResult, download, empty_data};
use crate::setup::{Outcome, install_one};

#[test]
fn a_verified_download_lands_at_its_destination() -> TestResult {
    let (_directory, data) = empty_data()?;
    let entry = download("models/sample.onnx", b"weights");
    let transport = FakeTransport::serving(&entry.url, b"weights".to_vec());
    assert_eq!(install_one(&entry, &data, &transport)?, Outcome::Installed);
    assert_eq!(fs::read(data.join("models/sample.onnx"))?, b"weights");
    assert!(!data.join("models/sample.onnx.partial").exists());
    Ok(())
}

#[test]
fn a_second_run_downloads_nothing() -> TestResult {
    let (_directory, data) = empty_data()?;
    let entry = download("models/sample.onnx", b"weights");
    let transport = FakeTransport::serving(&entry.url, b"weights".to_vec());
    install_one(&entry, &data, &transport)?;
    let outcome = install_one(&entry, &data, &transport)?;
    assert_eq!(outcome, Outcome::AlreadyPresent);
    assert_eq!(transport.requests.get(), 1);
    Ok(())
}

#[test]
fn a_corrupted_file_is_fetched_again() -> TestResult {
    let (_directory, data) = empty_data()?;
    let entry = download("models/sample.onnx", b"weights");
    fs::create_dir_all(data.join("models"))?;
    fs::write(data.join("models/sample.onnx"), b"damaged")?;
    let transport = FakeTransport::serving(&entry.url, b"weights".to_vec());
    assert_eq!(install_one(&entry, &data, &transport)?, Outcome::Installed);
    assert_eq!(fs::read(data.join("models/sample.onnx"))?, b"weights");
    Ok(())
}

#[test]
fn content_that_does_not_match_is_rejected_and_leaves_nothing() -> TestResult {
    let (_directory, data) = empty_data()?;
    let entry = download("models/sample.onnx", b"weights");
    let transport = FakeTransport::serving(&entry.url, b"tampered".to_vec());
    let error = install_one(&entry, &data, &transport)
        .err()
        .ok_or("accepted")?;
    assert!(error.to_string().contains("failed verification"));
    assert!(!data.join("models/sample.onnx").exists());
    assert!(!data.join("models/sample.onnx.partial").exists());
    Ok(())
}

#[test]
fn a_failing_transport_leaves_nothing() -> TestResult {
    let (_directory, data) = empty_data()?;
    let entry = download("models/sample.onnx", b"weights");
    let transport = FakeTransport::serving("http://elsewhere", Vec::new());
    assert!(install_one(&entry, &data, &transport).is_err());
    assert!(!data.join("models/sample.onnx.partial").exists());
    Ok(())
}

#[test]
fn a_destination_outside_the_data_directory_is_refused() -> TestResult {
    let (_directory, data) = empty_data()?;
    for destination in ["../escape", "/absolute"] {
        let entry = download(destination, b"x");
        let transport = FakeTransport::serving(&entry.url, b"x".to_vec());
        assert!(install_one(&entry, &data, &transport).is_err());
        assert_eq!(transport.requests.get(), 0);
    }
    Ok(())
}
