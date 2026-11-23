use std::fs;

use avatar_control::owned_browser::{BrowserFailure, OwnedBrowser};

use crate::rig::{
    Cleanup, EXITING, SERVING, SILENT, STUBBORN_SILENT, TestResult, check, fake_browser,
};

pub(crate) const URL: &str = "http://localhost:3000/?output=1";

pub(crate) fn failure_text(outcome: Result<i64, BrowserFailure>) -> String {
    outcome
        .err()
        .map(|failure| failure.to_string())
        .unwrap_or_default()
}

pub(crate) fn started(
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
