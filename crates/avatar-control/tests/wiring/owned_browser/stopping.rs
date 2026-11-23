use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};

use avatar_control::owned_browser::OwnedBrowser;

use crate::lifecycle::{URL, failure_text, started};
use crate::rig::{SERVING, STUBBORN, TestResult, check};

#[test]
pub(crate) fn stop_ends_the_owned_process_and_removes_the_pid_file() -> TestResult {
    let (_directory, browser, _cleanup) = started(SERVING)?;
    browser.start(URL)?;
    browser.stop()?;
    check(browser.owned_process()?.is_none(), "still owned")?;
    check(
        !browser.profile.join("browser.pid").exists(),
        "pid file kept",
    )
}

#[test]
pub(crate) fn stop_without_a_browser_is_quiet() -> TestResult {
    let (_directory, browser, _cleanup) = started(SERVING)?;
    fs::create_dir_all(&browser.profile)?;
    Ok(browser.stop()?)
}

#[test]
pub(crate) fn stop_reports_a_browser_that_ignores_the_signal() -> TestResult {
    let (_directory, mut browser, _cleanup) = started(STUBBORN)?;
    browser.start(URL)?;
    browser.timing.stop_attempts = 5;
    let outcome = browser.stop().err().map(|failure| failure.to_string());
    check(
        outcome.as_deref() == Some("The owned avatar browser did not stop"),
        format!("{outcome:?}"),
    )
}

#[test]
pub(crate) fn endpoint_answers_for_a_running_browser_and_times_out_otherwise() -> TestResult {
    let (_directory, mut browser, _cleanup) = started(SERVING)?;
    browser.timing.endpoint_attempts = 5;
    let missing = failure_text(browser.endpoint());
    browser.start(URL)?;
    check(missing == "The avatar browser has no endpoint", missing)?;
    check(browser.endpoint()? == 4242, "endpoint")
}

#[test]
pub(crate) fn controls_need_a_running_browser_and_then_open_the_control_page() -> TestResult {
    let (_directory, browser, _cleanup) = started(SERVING)?;
    fs::create_dir_all(&browser.profile)?;
    let refused = browser.controls().err().map(|failure| failure.to_string());
    check(
        refused.as_deref() == Some("Start the avatar before opening its controls"),
        format!("{refused:?}"),
    )?;
    browser.start(URL)?;
    Ok(browser.controls()?)
}

pub(crate) fn expect_port(port: i64) -> std::io::Result<()> {
    if port == 4242 {
        return Ok(());
    }
    Err(std::io::Error::other(format!("port {port}")))
}

#[test]
pub(crate) fn supervision_stops_the_browser_once_interrupted() -> TestResult {
    let (_directory, browser, _cleanup) = started(SERVING)?;
    let interrupted = AtomicBool::new(false);
    interrupted.store(true, Ordering::Release);
    browser.supervise(URL, &interrupted, expect_port)?;
    check(browser.owned_process()?.is_none(), "browser survived")
}

pub(crate) fn plant_stale_lock(browser: &OwnedBrowser) -> TestResult {
    fs::create_dir_all(&browser.profile)?;
    let target = format!("{}-99999999", browser.hostname);
    Ok(std::os::unix::fs::symlink(
        target,
        browser.profile.join("SingletonLock"),
    )?)
}

#[test]
pub(crate) fn stale_singleton_files_are_removed_before_a_new_browser_starts() -> TestResult {
    let (_directory, browser, _cleanup) = started(SERVING)?;
    plant_stale_lock(&browser)?;
    browser.start(URL)?;
    check(
        fs::symlink_metadata(browser.profile.join("SingletonLock")).is_err(),
        "lock kept",
    )
}
