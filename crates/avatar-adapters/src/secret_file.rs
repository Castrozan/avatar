use std::fs;
use std::io;
use std::path::Path;

pub fn read_trimmed(path: &Path) -> io::Result<String> {
    fs::read_to_string(path).map(|content| content.trim().to_owned())
}
