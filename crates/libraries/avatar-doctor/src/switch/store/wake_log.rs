use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub trait WakeLog {
    fn lines(&self) -> io::Result<Vec<String>>;
}

pub struct RotatedWakeLog {
    path: PathBuf,
}

impl RotatedWakeLog {
    #[must_use]
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    fn generations(&self) -> io::Result<Vec<PathBuf>> {
        let prefix = file_name_text(&self.path);
        let directory = self.path.parent().unwrap_or(Path::new("."));
        let mut paths = listing(directory)?;
        paths.retain(|path| is_generation(path, &prefix));
        paths.sort();
        Ok(paths)
    }
}

fn file_name_text(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn listing(directory: &Path) -> io::Result<Vec<PathBuf>> {
    fs::read_dir(directory)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect()
}

fn is_generation(path: &Path, prefix: &str) -> bool {
    path.file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with(prefix))
}

fn lines_of(content: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(content)
        .lines()
        .map(str::to_owned)
        .collect()
}

impl WakeLog for RotatedWakeLog {
    fn lines(&self) -> io::Result<Vec<String>> {
        let contents = self
            .generations()?
            .iter()
            .map(fs::read)
            .collect::<io::Result<Vec<Vec<u8>>>>()?;
        Ok(contents
            .iter()
            .flat_map(|content| lines_of(content))
            .collect())
    }
}
