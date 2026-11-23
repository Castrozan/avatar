use std::fs;
use std::os::unix::fs::DirBuilderExt;
use std::path::Path;
use std::process::Child;
use std::thread::sleep;

use super::OwnedBrowser;
use super::failure::BrowserFailure;
use super::profile::remove_if_present;
use super::spawn::spawn_browser;
use super::support::{read_port, runtime, signal_terminate, timeout};

fn prepare_profile(profile: &Path) -> Result<(), BrowserFailure> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(profile)?;
    Ok(())
}

pub(super) fn start(browser: &OwnedBrowser, url: &str) -> Result<i64, BrowserFailure> {
    prepare_profile(&browser.profile)?;
    let endpoint = browser.profile.join("DevToolsActivePort");
    if browser.owned_process()?.is_some() {
        return reuse_endpoint(&endpoint);
    }
    browser.remove_stale_locks()?;
    remove_if_present(&endpoint)?;
    launch(browser, url)
}

fn launch(browser: &OwnedBrowser, url: &str) -> Result<i64, BrowserFailure> {
    let child = spawn_browser(&browser.program, &browser.profile, url)?;
    fs::write(browser.profile.join("browser.pid"), child.id().to_string())?;
    wait_for_endpoint(browser, child)
}

fn reuse_endpoint(endpoint: &Path) -> Result<i64, BrowserFailure> {
    if !endpoint.exists() {
        return Err(runtime(
            "The owned avatar browser has no endpoint; stop it before restarting",
        ));
    }
    read_port(endpoint)
}

fn wait_for_endpoint(browser: &OwnedBrowser, mut child: Child) -> Result<i64, BrowserFailure> {
    let attempts = 0..browser.timing.endpoint_attempts;
    let outcome = attempts
        .into_iter()
        .find_map(|_| poll_then_pause(browser, &mut child).transpose());
    match outcome {
        Some(result) => result,
        None => give_up(&child),
    }
}

fn give_up(child: &Child) -> Result<i64, BrowserFailure> {
    signal_terminate(i64::from(child.id()))?;
    Err(timeout(
        "The avatar browser did not expose its own endpoint",
    ))
}

fn poll_then_pause(
    browser: &OwnedBrowser,
    child: &mut Child,
) -> Result<Option<i64>, BrowserFailure> {
    let polled = poll_endpoint(browser, child)?;
    if polled.is_none() {
        sleep(browser.timing.poll_interval);
    }
    Ok(polled)
}

fn ensure_running(child: &mut Child) -> Result<(), BrowserFailure> {
    if child.try_wait()?.is_some() {
        return Err(runtime(
            "The avatar browser did not start a new owned instance",
        ));
    }
    Ok(())
}

fn poll_endpoint(browser: &OwnedBrowser, child: &mut Child) -> Result<Option<i64>, BrowserFailure> {
    ensure_running(child)?;
    let endpoint = browser.profile.join("DevToolsActivePort");
    if endpoint.exists() {
        record_owner(browser, i64::from(child.id()))?;
        return read_port(&endpoint).map(Some);
    }
    Ok(None)
}

fn record_owner(browser: &OwnedBrowser, fallback: i64) -> Result<(), BrowserFailure> {
    let identifier = browser.owned_process()?.unwrap_or(fallback);
    Ok(fs::write(
        browser.profile.join("browser.pid"),
        identifier.to_string(),
    )?)
}
