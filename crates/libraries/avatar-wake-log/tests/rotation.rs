use std::collections::BTreeMap;
use std::error::Error;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use avatar_wake_log::{RotationPolicy, WakeLog, wake_log_path};
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/rotation.json");

#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
    paths: Vec<PathCase>,
}

#[derive(Deserialize)]
struct Case {
    directory_mode: u32,
    lines: Vec<String>,
    max_bytes: u64,
    backups: u32,
    files: BTreeMap<String, Stored>,
}

#[derive(Deserialize)]
struct Stored {
    text: String,
    mode: u32,
}

#[derive(Deserialize)]
struct PathCase {
    state_home: Option<String>,
    home: String,
    path: String,
}

fn mode_of(path: &Path) -> Result<u32, Box<dyn Error>> {
    Ok(std::fs::metadata(path)?.permissions().mode() & 0o777)
}

fn write_lines(directory: &Path, case: &Case) -> Result<(), Box<dyn Error>> {
    let policy = RotationPolicy {
        maximum_bytes: case.max_bytes,
        backups: case.backups,
    };
    let mut log = WakeLog::open(directory.join("wake.log"), policy)?;
    case.lines.iter().try_for_each(|line| log.info(line))?;
    Ok(())
}

fn replay(case: &Case) -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    let directory = root.path().join("state").join("deep");
    write_lines(&directory, case)?;
    assert_eq!(mode_of(&directory)?, case.directory_mode);
    assert_files(&directory, case)
}

fn file_names(directory: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut names = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        names.push(entry?.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    Ok(names)
}

fn assert_stored(path: &Path, stored: &Stored) -> Result<(), Box<dyn Error>> {
    assert_eq!(std::fs::read_to_string(path)?, stored.text);
    assert_eq!(mode_of(path)?, stored.mode);
    Ok(())
}

fn assert_files(directory: &Path, case: &Case) -> Result<(), Box<dyn Error>> {
    let expected: Vec<String> = case.files.keys().cloned().collect();
    assert_eq!(file_names(directory)?, expected);
    for (name, stored) in &case.files {
        assert_stored(&directory.join(name), stored)?;
    }
    Ok(())
}

#[test]
fn rotation_matches_the_python_rotating_handler() -> Result<(), Box<dyn Error>> {
    let fixture: Fixture = serde_json::from_str(FIXTURE)?;
    assert_eq!(fixture.cases.len(), 42);
    fixture.cases.iter().try_for_each(replay)
}

#[test]
fn the_log_location_follows_the_state_home() -> Result<(), Box<dyn Error>> {
    let fixture: Fixture = serde_json::from_str(FIXTURE)?;
    for case in &fixture.paths {
        let path = wake_log_path(case.state_home.as_deref(), Path::new(&case.home));
        assert_eq!(path.to_string_lossy(), case.path);
    }
    Ok(())
}
