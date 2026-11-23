use std::path::{Path, PathBuf};

use super::units::SessionUnits;

pub const LISTENER_SERVICE: &str = "avatar-listener.service";
pub const LIVE_SERVICE: &str = "avatar-live.service";
pub const CAPTURE_SERVICES: [&str; 2] = [LISTENER_SERVICE, LIVE_SERVICE];
pub const MAC_RUNTIME_DIRECTORY: &str = "Library/Caches/avatar";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Mac,
    Linux,
}

pub struct RuntimeEnvironment<'a> {
    pub platform: Platform,
    pub home: &'a Path,
    pub runtime_dir: Option<&'a str>,
    pub user_identifier: u32,
}

#[must_use]
pub fn runtime_directory(environment: &RuntimeEnvironment) -> PathBuf {
    if environment.platform == Platform::Mac {
        return environment.home.join(MAC_RUNTIME_DIRECTORY);
    }
    let base = environment.runtime_dir.map_or_else(
        || format!("/run/user/{}", environment.user_identifier),
        str::to_owned,
    );
    Path::new(&base).join("avatar")
}

pub fn capturing_services(units: &mut impl SessionUnits) -> Vec<&'static str> {
    CAPTURE_SERVICES
        .into_iter()
        .filter(|service| units.is_active(service))
        .collect()
}

pub fn listening(units: &mut impl SessionUnits) -> bool {
    CAPTURE_SERVICES
        .into_iter()
        .any(|service| units.is_active(service))
}
