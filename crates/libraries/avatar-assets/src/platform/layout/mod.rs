use std::path::{Path, PathBuf};

use crate::platform::environment::Environment;

const OVERRIDE_VARIABLE: &str = "AVATAR_DATA_DIRECTORY";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDirectory(PathBuf);

impl DataDirectory {
    #[must_use]
    pub fn at(path: PathBuf) -> Self {
        Self(path)
    }

    #[must_use]
    pub fn locate(environment: &impl Environment) -> Self {
        let chosen = environment
            .variable(OVERRIDE_VARIABLE)
            .map_or_else(|| platform_default(environment), PathBuf::from);
        Self(chosen)
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }

    #[must_use]
    pub fn join(&self, relative: &str) -> PathBuf {
        self.0.join(relative)
    }
}

fn platform_default(environment: &impl Environment) -> PathBuf {
    let home = environment.variable("HOME").map(PathBuf::from);
    let share = environment.variable("XDG_DATA_HOME").map(PathBuf::from);
    default_for(cfg!(target_os = "macos"), home, share)
}

fn default_for(macos: bool, home: Option<PathBuf>, share: Option<PathBuf>) -> PathBuf {
    let home = home.unwrap_or_default();
    if macos {
        return home.join("Library/Application Support/avatar");
    }
    share
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| home.join(".local/share"))
        .join("avatar")
}

#[cfg(test)]
mod tests;
