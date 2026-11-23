use std::fs;

use super::support::{
    FakeTransport, TestResult, archive_download, empty_data, tarball, tarball_with_raw_path,
};
use crate::setup::{Outcome, install_one};

#[test]
fn an_archive_is_unpacked_without_its_top_directory() -> TestResult {
    let (_directory, data) = empty_data()?;
    let body = tarball(&[("package/build/a.js", "alpha"), ("package/b.js", "beta")])?;
    let entry = archive_download("vendor/three", &body);
    let transport = FakeTransport::serving(&entry.url, body);
    assert_eq!(install_one(&entry, &data, &transport)?, Outcome::Installed);
    assert_eq!(fs::read(data.join("vendor/three/build/a.js"))?, b"alpha");
    assert_eq!(fs::read(data.join("vendor/three/b.js"))?, b"beta");
    assert!(!data.join("vendor/three.partial").exists());
    Ok(())
}

#[test]
fn an_unpacked_archive_is_not_fetched_again() -> TestResult {
    let (_directory, data) = empty_data()?;
    let body = tarball(&[("package/a.js", "alpha")])?;
    let entry = archive_download("vendor/three", &body);
    let transport = FakeTransport::serving(&entry.url, body);
    install_one(&entry, &data, &transport)?;
    let again = install_one(&entry, &data, &transport)?;
    assert_eq!(again, Outcome::AlreadyPresent);
    assert_eq!(transport.requests.get(), 1);
    Ok(())
}

#[test]
fn a_reinstalled_archive_drops_files_the_new_version_lacks() -> TestResult {
    let (_directory, data) = empty_data()?;
    let stale = data.join("vendor/three/stale.js");
    fs::create_dir_all(data.join("vendor/three"))?;
    fs::write(&stale, "old")?;
    let body = tarball(&[("package/a.js", "alpha")])?;
    let entry = archive_download("vendor/three", &body);
    install_one(&entry, &data, &FakeTransport::serving(&entry.url, body))?;
    assert!(!stale.exists());
    Ok(())
}

#[test]
fn an_archive_member_climbing_out_is_refused() -> TestResult {
    let (_directory, data) = empty_data()?;
    let body = tarball_with_raw_path("package/../../escape.js", "bad")?;
    let entry = archive_download("vendor/three", &body);
    let transport = FakeTransport::serving(&entry.url, body);
    assert!(install_one(&entry, &data, &transport).is_err());
    assert!(!data.join("escape.js").exists());
    Ok(())
}
