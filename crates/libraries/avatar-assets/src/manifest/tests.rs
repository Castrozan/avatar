use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn the_embedded_manifest_names_every_download_once() -> TestResult {
    let downloads = embedded_downloads()?;
    let mut names: Vec<_> = downloads.iter().map(|download| &download.name).collect();
    names.sort();
    names.dedup();
    assert!(downloads.len() > 20);
    assert_eq!(names.len(), downloads.len());
    Ok(())
}

fn is_secure(entry: &Download) -> bool {
    entry.url.starts_with("https://")
}

fn is_relative(entry: &Download) -> bool {
    !entry.destination.starts_with('/')
}

#[test]
fn every_embedded_download_pins_a_digest() -> TestResult {
    let downloads = embedded_downloads()?;
    downloads
        .iter()
        .try_for_each(|entry| crate::integrity::Integrity::parse(&entry.integrity).map(drop))?;
    Ok(())
}

#[test]
fn every_embedded_download_is_https_and_relative() -> TestResult {
    let downloads = embedded_downloads()?;
    assert!(downloads.iter().all(is_secure));
    assert!(downloads.iter().all(is_relative));
    Ok(())
}
