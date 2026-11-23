use std::io;
use std::path::PathBuf;

use avatar_seat::local::SeatConfig;

const HOME: &str = "HOME";
const AVATAR_HOST: &str = "AVATAR_HOST";
const BROWSER_APPLICATION: &str = "AVATAR_BROWSER_APPLICATION";
pub(super) const SEAT_PAGE: &str = "AVATAR_SEAT_PAGE";
pub(super) const WAKE_PAGE: &str = "AVATAR_WAKE_PAGE";
pub(super) const CAMERA_START_SCRIPT: &str = "AVATAR_CAMERA_START_SCRIPT";
const SEAT_HOLDER_PYTHON: &str = "AVATAR_SEAT_HOLDER_PYTHON";
const SEAT_HOLDER: &str = "AVATAR_SEAT_HOLDER";

pub(super) fn required(name: &str) -> io::Result<String> {
    std::env::var(name).map_err(|_| io::Error::other(format!("{name} is not set")))
}

pub(super) fn page_directory(name: &str) -> io::Result<PathBuf> {
    required(name).map(PathBuf::from)
}

pub(crate) fn seat_config() -> io::Result<SeatConfig> {
    Ok(SeatConfig {
        avatar_host: required(AVATAR_HOST)?,
        browser_application: required(BROWSER_APPLICATION)?,
        home: required(HOME)?,
        python: String::new(),
        holder_path: String::new(),
    })
}

pub(super) fn holder_config() -> io::Result<SeatConfig> {
    Ok(SeatConfig {
        python: required(SEAT_HOLDER_PYTHON)?,
        holder_path: required(SEAT_HOLDER)?,
        ..seat_config()?
    })
}
