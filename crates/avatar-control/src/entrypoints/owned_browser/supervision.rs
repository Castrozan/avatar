use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;

use super::OwnedBrowser;
use super::failure::BrowserFailure;
use super::spawn::open_controls as spawn_controls;

pub(super) fn open_controls(browser: &OwnedBrowser) -> Result<(), BrowserFailure> {
    if browser.owned_process()?.is_none() {
        let text = "Start the avatar before opening its controls";
        return Err(BrowserFailure::Runtime(text.to_owned()));
    }
    spawn_controls(&browser.program, &browser.profile)?;
    Ok(())
}

pub(super) fn supervise(
    browser: &OwnedBrowser,
    url: &str,
    interrupted: &AtomicBool,
    announce: impl FnOnce(i64) -> io::Result<()>,
) -> Result<(), BrowserFailure> {
    announce(browser.start(url)?)?;
    while !interrupted.load(Ordering::Acquire) && browser.owned_process()?.is_some() {
        sleep(browser.timing.supervision_interval);
    }
    if interrupted.load(Ordering::Acquire) {
        return browser.stop();
    }
    Ok(())
}
