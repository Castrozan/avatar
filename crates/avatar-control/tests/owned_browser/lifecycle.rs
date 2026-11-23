use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};

use avatar_control::owned_browser::{BrowserFailure, OwnedBrowser};

use crate::rig::{
    Cleanup, EXITING, SERVING, SILENT, STUBBORN, STUBBORN_SILENT, TestResult, check, fake_browser,
};

const URL: &str = "http://localhost:3000/?output=1";

fn failure_text(outcome: Result<i64, BrowserFailure>) -> String {
    outcome
        .err()
        .map(|failure| failure.to_string())
        .unwrap_or_default()
}

fn started(
    body: &str,
) -> Result<(tempfile::TempDir, OwnedBrowser, Cleanup), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let browser = fake_browser(directory.path(), body)?;
    let cleanup = Cleanup(browser.profile.clone());
    Ok((directory, browser, cleanup))
}

#[test]
fn start_prints_the_endpoint_and_a_second_start_reuses_the_browser() -> TestResult {
    let (_directory, browser, _cleanup) = started(SERVING)?;
    let first = browser.start(URL)?;
    let owner = browser.owned_process()?;
    let second = browser.start(URL)?;
    check(first == 4242 && second == 4242, format!("{first} {second}"))?;
    check(owner == browser.owned_process()?, "owner changed")?;
    check(owner.is_some(), "no owner")
}

fn expected_arguments(browser: &OwnedBrowser) -> String {
    let profile = browser.profile.display();
    let options = "--class=avatar-browser\n--remote-debugging-port=0\n--autoplay-policy=no-user-gesture-required\n--use-fake-ui-for-media-stream\n--test-type\n--disable-background-timer-throttling\n--disable-renderer-backgrounding\n--disable-backgrounding-occluded-windows\n--ozone-platform=x11";
    format!("--user-data-dir={profile}\n{options}\n--app={URL}\n")
}

#[test]
fn the_browser_is_started_with_the_python_arguments() -> TestResult {
    let (_directory, browser, _cleanup) = started(SERVING)?;
    browser.start(URL)?;
    let recorded = fs::read_to_string(browser.profile.join("arguments.txt"))?;
    check(recorded == expected_arguments(&browser), recorded)
}

#[test]
fn a_running_browser_without_an_endpoint_refuses_a_restart() -> TestResult {
    let (_directory, mut browser, _cleanup) = started(STUBBORN_SILENT)?;
    browser.timing.endpoint_attempts = 3;
    let first = failure_text(browser.start(URL));
    let second = failure_text(browser.start(URL));
    let expected = "The owned avatar browser has no endpoint; stop it before restarting";
    check(
        first == "The avatar browser did not expose its own endpoint",
        first,
    )?;
    check(second == expected, second)
}

#[test]
fn a_browser_that_exits_straight_away_is_reported() -> TestResult {
    let (_directory, browser, _cleanup) = started(EXITING)?;
    let text = failure_text(browser.start(URL));
    check(
        text == "The avatar browser did not start a new owned instance",
        text,
    )
}

#[test]
fn a_browser_that_never_exposes_an_endpoint_times_out_and_is_terminated() -> TestResult {
    let (_directory, mut browser, _cleanup) = started(SILENT)?;
    browser.timing.endpoint_attempts = 10;
    let text = failure_text(browser.start(URL));
    check(
        text == "The avatar browser did not expose its own endpoint",
        text,
    )
}

#[test]
fn stop_ends_the_owned_process_and_removes_the_pid_file() -> TestResult {
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
fn stop_without_a_browser_is_quiet() -> TestResult {
    let (_directory, browser, _cleanup) = started(SERVING)?;
    fs::create_dir_all(&browser.profile)?;
    Ok(browser.stop()?)
}

#[test]
fn stop_reports_a_browser_that_ignores_the_signal() -> TestResult {
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
fn endpoint_answers_for_a_running_browser_and_times_out_otherwise() -> TestResult {
    let (_directory, mut browser, _cleanup) = started(SERVING)?;
    browser.timing.endpoint_attempts = 5;
    let missing = failure_text(browser.endpoint());
    browser.start(URL)?;
    check(missing == "The avatar browser has no endpoint", missing)?;
    check(browser.endpoint()? == 4242, "endpoint")
}

#[test]
fn controls_need_a_running_browser_and_then_open_the_control_page() -> TestResult {
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

fn expect_port(port: i64) -> std::io::Result<()> {
    if port == 4242 {
        return Ok(());
    }
    Err(std::io::Error::other(format!("port {port}")))
}

#[test]
fn supervision_stops_the_browser_once_interrupted() -> TestResult {
    let (_directory, browser, _cleanup) = started(SERVING)?;
    let interrupted = AtomicBool::new(false);
    interrupted.store(true, Ordering::Release);
    browser.supervise(URL, &interrupted, expect_port)?;
    check(browser.owned_process()?.is_none(), "browser survived")
}

fn plant_stale_lock(browser: &OwnedBrowser) -> TestResult {
    fs::create_dir_all(&browser.profile)?;
    let target = format!("{}-99999999", browser.hostname);
    Ok(std::os::unix::fs::symlink(
        target,
        browser.profile.join("SingletonLock"),
    )?)
}

#[test]
fn stale_singleton_files_are_removed_before_a_new_browser_starts() -> TestResult {
    let (_directory, browser, _cleanup) = started(SERVING)?;
    plant_stale_lock(&browser)?;
    browser.start(URL)?;
    check(
        fs::symlink_metadata(browser.profile.join("SingletonLock")).is_err(),
        "lock kept",
    )
}
