use std::error::Error;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub(super) type TestResult = Result<(), Box<dyn Error>>;

pub(super) struct Scratch {
    directory: PathBuf,
}

impl Scratch {
    pub(super) fn new(test_name: &str) -> io::Result<Self> {
        let directory = std::env::temp_dir()
            .join("avatar-camera-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory)?;
        Ok(Self { directory })
    }

    pub(super) fn file(&self, name: &str) -> String {
        self.directory.join(name).to_string_lossy().into_owned()
    }

    pub(super) fn fake_encoder(&self, body: &str) -> io::Result<String> {
        let path = self.directory.join("encoder");
        fs::write(&path, format!("#!/bin/sh\n{body}\n"))?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
        Ok(path.to_string_lossy().into_owned())
    }
}

pub(super) fn recorded_arguments(path: &str) -> io::Result<Vec<String>> {
    let bytes = fs::read(path)?;
    let mut arguments: Vec<String> = bytes
        .split(|byte| *byte == 0)
        .map(|field| String::from_utf8_lossy(field).into_owned())
        .collect();
    arguments.pop();
    Ok(arguments)
}
