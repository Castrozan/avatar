use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use avatar_adapters::wake_log::WakeLogFile;
use serde_json::Value;

use super::{section, text};

#[test]
fn rotation_matches_python_rotating_handler_byte_for_byte() -> Result<(), Box<dyn Error>> {
    for case in section("rotation") {
        replay(&case)?;
    }
    Ok(())
}

fn replay(case: &Value) -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("state").join("wake.log");
    append_all(&path, case)?;
    assert_files_match(&path, &case["files"])?;
    assert_eq!(fs::metadata(&path)?.permissions().mode() & 0o777, 0o600);
    Ok(())
}

fn append_all(path: &Path, case: &Value) -> Result<(), Box<dyn Error>> {
    let backups = u32::try_from(case["backups"].as_u64().unwrap_or(0))?;
    let log = WakeLogFile::open(path, case["max_bytes"].as_u64().unwrap_or(0), backups)?;
    for line in case["lines"].as_array().into_iter().flatten() {
        log.append(text(line))?;
    }
    Ok(())
}

fn assert_files_match(path: &Path, expected: &Value) -> Result<(), Box<dyn Error>> {
    let expected = expected.as_object().cloned().unwrap_or_default();
    let present = fs::read_dir(path.parent().unwrap_or(path))?.count();
    assert_eq!(present, expected.len());
    for (name, content) in &expected {
        assert_eq!(
            fs::read(path.with_file_name(name))?,
            text(content).as_bytes()
        );
    }
    Ok(())
}
