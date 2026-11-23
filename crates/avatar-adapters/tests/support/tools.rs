use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use avatar_adapters::commands::Programs;
use tempfile::TempDir;

pub(crate) type TestResult = Result<(), Box<dyn Error>>;

pub(crate) struct Tools {
    directory: TempDir,
}

impl Tools {
    pub(crate) fn with(scripts: &[(&str, &str)]) -> Result<Self, Box<dyn Error>> {
        let directory = tempfile::tempdir()?;
        let tools = Self { directory };
        scripts
            .iter()
            .try_for_each(|(name, body)| tools.write(name, body))?;
        Ok(tools)
    }

    pub(crate) fn write(&self, name: &str, body: &str) -> TestResult {
        let path = self.directory.path().join(name);
        fs::write(&path, format!("#!/bin/sh\n{body}\n"))?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
        Ok(())
    }

    pub(crate) fn programs(&self) -> Programs {
        Programs::in_directory(self.directory.path())
    }

    pub(crate) fn path(&self) -> &Path {
        self.directory.path()
    }

    pub(crate) fn read(&self, name: &str) -> String {
        fs::read_to_string(self.path().join(name)).unwrap_or_default()
    }
}
