use std::path::PathBuf;

use avatar_listening::listener::runtime::{Platform, RuntimeEnvironment, runtime_directory};
use nix::unistd::getuid;

const fn platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::Mac
    } else {
        Platform::Linux
    }
}

#[must_use]
pub(crate) fn session_directory() -> PathBuf {
    let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_default();
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").ok();
    runtime_directory(&RuntimeEnvironment {
        platform: platform(),
        home: &home,
        runtime_dir: runtime_dir.as_deref(),
        user_identifier: getuid().as_raw(),
    })
}
