use std::error::Error;
use std::fs;
use std::future::Future;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use avatar_control::playback::{AudioPlayer, PlaybackReport};

pub(super) type TestResult = Result<(), Box<dyn Error>>;

pub(super) const CLIP_PATH: &str = "/tmp/fixture/clip.wav";

pub(super) struct Scratch {
    directory: PathBuf,
}

impl Scratch {
    pub(super) fn new(test_name: &str) -> io::Result<Self> {
        let directory = std::env::temp_dir()
            .join("avatar-playback-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory)?;
        Ok(Self { directory })
    }

    fn path(&self, name: &str) -> PathBuf {
        self.directory.join(name)
    }

    pub(super) fn marker_path(&self) -> PathBuf {
        self.path("marker")
    }

    pub(super) fn fake_program(&self, name: &str, body: &str) -> io::Result<String> {
        let path = self.path(name);
        let script = format!(
            "#!/bin/sh\nprintf '%s\\0' \"$@\" > {arguments}\nenv > {environment}\ncat > {stdin}\n{body}\n",
            arguments = self.path("arguments.bin").display(),
            environment = self.path("environment.txt").display(),
            stdin = self.path("stdin.bin").display(),
        );
        fs::write(&path, script)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
        Ok(path.to_string_lossy().into_owned())
    }

    pub(super) fn recorded_arguments(&self) -> io::Result<Vec<String>> {
        let bytes = fs::read(self.path("arguments.bin"))?;
        let mut arguments: Vec<String> = bytes
            .split(|byte| *byte == 0)
            .map(|field| String::from_utf8_lossy(field).into_owned())
            .collect();
        arguments.pop();
        Ok(arguments)
    }

    pub(super) fn recorded_environment(&self) -> io::Result<String> {
        fs::read_to_string(self.path("environment.txt"))
    }

    pub(super) fn recorded_stdin(&self) -> io::Result<Vec<u8>> {
        fs::read(self.path("stdin.bin"))
    }
}

pub(super) async fn within_limit<Output>(
    future: impl Future<Output = Output>,
) -> Result<Output, Box<dyn Error>> {
    Ok(tokio::time::timeout(Duration::from_secs(10), future).await?)
}

pub(super) fn block_on<Output>(
    future: impl Future<Output = Output>,
) -> Result<Output, Box<dyn Error>> {
    Ok(tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(future))
}

pub(super) fn has_environment_line(environment: &str, line: &str) -> bool {
    environment.lines().any(|candidate| candidate == line)
}

pub(super) fn clip() -> &'static Path {
    Path::new(CLIP_PATH)
}

pub(super) async fn play_then_stop(
    player: &impl AudioPlayer,
) -> Result<PlaybackReport, Box<dyn Error>> {
    let mut playback = player.play(clip());
    playback.stop();
    within_limit(playback.finished()).await
}

pub(super) fn killed_by_signal(report: &PlaybackReport) -> bool {
    report
        .exit_status
        .is_some_and(|status| status.code().is_none())
}
