use std::fs;
use std::os::unix::fs::DirBuilderExt;
use std::path::Path;
use std::process::Child;
use std::thread::sleep;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use super::OwnedBrowser;
use super::failure::BrowserFailure;
use super::profile::{remove_if_present, remove_stale_locks};
use super::spawn::spawn_browser;

fn runtime(text: &str) -> BrowserFailure {
    BrowserFailure::Runtime(text.to_owned())
}

fn timeout(text: &str) -> BrowserFailure {
    BrowserFailure::Timeout(text.to_owned())
}

fn signal_terminate(identifier: i64) -> Result<(), BrowserFailure> {
    let pid = i32::try_from(identifier)
        .map_err(|_| BrowserFailure::Value("pid out of range".to_owned()))?;
    kill(Pid::from_raw(pid), Signal::SIGTERM).map_err(|errno| std::io::Error::from(errno).into())
}

fn read_port(path: &Path) -> Result<i64, BrowserFailure> {
    let text = fs::read_to_string(path)?;
    let line = text
        .lines()
        .next()
        .ok_or_else(|| BrowserFailure::Value("list index out of range".to_owned()))?;
    line.trim()
        .parse()
        .map_err(|_| BrowserFailure::Value(format!("invalid literal for int(): {line:?}")))
}

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
