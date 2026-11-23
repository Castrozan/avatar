use std::path::Path;

use crate::support::{Outcome, text};
use serde_json::Value;

fn assert_stored_file(path: &Path, file: &Value) -> Outcome {
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(std::fs::read_to_string(path)?, text(&file["text"])?);
    let mode = u64::from(std::fs::metadata(path)?.permissions().mode() & 0o777);
    assert_eq!(Some(mode), file["mode"].as_u64());
    Ok(())
}

pub(super) fn assert_voice_files(root: &Path, scenario: &Value) -> Outcome {
    let directory = root.join("state/avatar/voices");
    let expected = scenario["voices"].as_object().ok_or("voices")?;
    for (name, file) in expected {
        assert_stored_file(&directory.join(name), file)?;
    }
    let present = std::fs::read_dir(&directory).map_or(0, Iterator::count);
    assert_eq!(present, expected.len());
    Ok(())
}
