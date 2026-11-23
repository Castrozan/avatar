use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use serde_json::Value;

use crate::support::TestResult;

fn latin1_bytes(text: &str) -> Vec<u8> {
    text.chars()
        .filter_map(|character| u8::try_from(u32::from(character)).ok())
        .collect()
}

fn write_files(root: &Path, files: &Value) -> TestResult {
    for (name, content) in files.as_object().into_iter().flatten() {
        let path = root.join(name);
        fs::create_dir_all(path.parent().ok_or("no parent")?)?;
        fs::write(path, latin1_bytes(content.as_str().unwrap_or_default()))?;
    }
    Ok(())
}

fn write_directories(root: &Path, fixture: &Value) -> TestResult {
    for name in fixture["dirs"].as_array().into_iter().flatten() {
        fs::create_dir_all(root.join(name.as_str().unwrap_or_default()))?;
    }
    for name in fixture["listing"].as_array().into_iter().flatten() {
        fs::write(
            root.join("listing").join(name.as_str().unwrap_or_default()),
            "1",
        )?;
    }
    Ok(())
}

pub(crate) fn build(root: &Path, fixture: &Value) -> TestResult {
    write_files(root, &fixture["files"])?;
    write_directories(root, fixture)?;
    for (name, target) in fixture["symlinks"].as_object().into_iter().flatten() {
        symlink(target.as_str().unwrap_or_default(), root.join(name))?;
    }
    Ok(symlink("b.txt", root.join("listing").join("sym.txt"))?)
}
