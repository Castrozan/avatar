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
        let tools = Self {
            directory: tempfile::tempdir()?,
        };
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
}

pub(crate) fn words(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}
