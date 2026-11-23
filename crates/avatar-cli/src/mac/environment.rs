use std::io;
use std::path::PathBuf;

use avatar_assets::{Asset, resolve};
use avatar_seat::local::SeatConfig;

const HOME: &str = "HOME";
const AVATAR_HOST: &str = "AVATAR_HOST";
const BROWSER_APPLICATION: &str = "AVATAR_BROWSER_APPLICATION";

pub(super) fn required(name: &str) -> io::Result<String> {
    std::env::var(name).map_err(|_| io::Error::other(format!("{name} is not set")))
}

pub(super) fn page_directory(asset: Asset) -> io::Result<PathBuf> {
    resolve(asset)
}

pub(crate) fn seat_config() -> io::Result<SeatConfig> {
    Ok(SeatConfig {
        avatar_host: required(AVATAR_HOST)?,
        browser_application: required(BROWSER_APPLICATION)?,
        home: required(HOME)?,
        holder_program: std::env::current_exe()?.to_string_lossy().into_owned(),
    })
}
