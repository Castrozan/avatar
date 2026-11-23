use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use avatar_control::owned_browser::{BrowserFailure, OwnedBrowser, profile_directory};
use clap::{Args, Subcommand};
use tokio::runtime::Builder;

use crate::interruption::Interruption;

#[derive(Args)]
pub(crate) struct BrowserOptions {
    #[command(subcommand)]
    action: BrowserAction,
}

#[derive(Subcommand)]
enum BrowserAction {
    #[command(about = "Start the owned browser on URL and keep supervising it")]
    Supervise { url: String },
    #[command(about = "Print the debugging port of the owned browser")]
    Endpoint,
    #[command(about = "Stop the owned browser")]
    Stop,
    #[command(about = "Open the avatar controls in the owned browser profile")]
    Controls,
}

fn owned_browser() -> io::Result<OwnedBrowser> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::other("HOME is not set"))?;
    let data_home = std::env::var("XDG_DATA_HOME").ok();
    OwnedBrowser::for_profile(profile_directory(data_home.as_deref(), &home))
}

fn failed(failure: BrowserFailure) -> io::Error {
    io::Error::other(failure)
}

fn announce(port: i64) -> io::Result<()> {
    writeln!(io::stdout().lock(), "{port}")
}

pub(crate) fn run(options: &BrowserOptions) -> io::Result<()> {
    let browser = owned_browser()?;
    match &options.action {
        BrowserAction::Supervise { url } => supervise(&browser, url),
        BrowserAction::Endpoint => announce(browser.endpoint().map_err(failed)?),
        BrowserAction::Stop => browser.stop().map_err(failed),
        BrowserAction::Controls => browser.controls().map_err(failed),
    }
}

fn supervise(browser: &OwnedBrowser, url: &str) -> io::Result<()> {
    let interrupted = Arc::new(AtomicBool::new(false));
    let runtime = Builder::new_multi_thread().enable_all().build()?;
    let watcher = runtime.block_on(async { Interruption::listen() })?;
    runtime.spawn(raise_on_interruption(watcher, Arc::clone(&interrupted)));
    browser
        .supervise(url, &interrupted, announce)
        .map_err(failed)
}

async fn raise_on_interruption(mut watcher: Interruption, interrupted: Arc<AtomicBool>) {
    watcher.arrived().await;
    interrupted.store(true, Ordering::Release);
}
