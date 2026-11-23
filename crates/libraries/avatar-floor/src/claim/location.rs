use std::fmt::Write;
use std::path::{Path, PathBuf};

const CLAIMS_DIRECTORY: &str = "wake-claims";

pub(super) fn claims_directory(directory: &Path) -> PathBuf {
    directory.join(CLAIMS_DIRECTORY)
}

#[must_use]
pub fn claim_path(directory: &Path, pane: &str) -> PathBuf {
    claims_directory(directory).join(format!("pane-{}", percent_encode(pane)))
}

fn percent_encode(text: &str) -> String {
    let mut encoded = String::new();
    for byte in text.bytes() {
        push_encoded_byte(&mut encoded, byte);
    }
    encoded
}

fn push_encoded_byte(encoded: &mut String, byte: u8) {
    if byte.is_ascii_alphanumeric() || b"_.-~".contains(&byte) {
        encoded.push(char::from(byte));
    } else {
        let _ = write!(encoded, "%{byte:02X}");
    }
}
