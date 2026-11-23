use std::thread::sleep;

use super::OwnedBrowser;
use super::failure::BrowserFailure;
use super::profile::{remove_if_present, remove_stale_locks};
use super::support::{read_port, signal_terminate, timeout};

pub(super) fn endpoint(browser: &OwnedBrowser) -> Result<i64, BrowserFailure> {
    let attempts = 0..browser.timing.endpoint_attempts;
    let outcome = attempts
        .into_iter()
        .find_map(|_| endpoint_ready(browser).transpose());
    outcome.unwrap_or_else(|| Err(timeout("The avatar browser has no endpoint")))
}

fn endpoint_ready(browser: &OwnedBrowser) -> Result<Option<i64>, BrowserFailure> {
    let path = browser.profile.join("DevToolsActivePort");
    if browser.owned_process()?.is_some() && path.exists() {
        return read_port(&path).map(Some);
    }
    sleep(browser.timing.poll_interval);
    Ok(None)
}

pub(super) fn stop(browser: &OwnedBrowser) -> Result<(), BrowserFailure> {
    if let Some(identifier) = browser.owned_process()? {
        signal_terminate(identifier)?;
        wait_until_stopped(browser, identifier)?;
    }
    remove_if_present(&browser.profile.join("browser.pid"))?;
    remove_stale_locks(&browser.profile, &browser.proc_root, &browser.hostname)
}

fn wait_until_stopped(browser: &OwnedBrowser, identifier: i64) -> Result<(), BrowserFailure> {
    let attempts = 0..browser.timing.stop_attempts;
    let outcome = attempts
        .into_iter()
        .find_map(|_| stopped_or_pause(browser, identifier).transpose());
    outcome.unwrap_or_else(|| Err(timeout("The owned avatar browser did not stop")))
}

fn stopped_or_pause(browser: &OwnedBrowser, identifier: i64) -> Result<Option<()>, BrowserFailure> {
    if browser.owned_process()? != Some(identifier) {
        return Ok(Some(()));
    }
    sleep(browser.timing.poll_interval);
    Ok(None)
}
