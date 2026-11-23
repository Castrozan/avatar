use std::path::{Path, PathBuf};

fn hexadecimal_value(byte: u8) -> Option<u8> {
    char::from(byte)
        .to_digit(16)
        .and_then(|digit| u8::try_from(digit).ok())
}

fn escaped_byte(pair: &[u8]) -> Option<u8> {
    match pair {
        [high, low] => Some(hexadecimal_value(*high)? * 16 + hexadecimal_value(*low)?),
        _ => None,
    }
}

fn decode_at(bytes: &[u8], position: usize) -> (u8, usize) {
    let byte = bytes.get(position).copied().unwrap_or_default();
    if byte != b'%' {
        return (byte, 1);
    }
    match bytes.get(position + 1..position + 3).and_then(escaped_byte) {
        Some(decoded) => (decoded, 3),
        None => (byte, 1),
    }
}

#[must_use]
pub fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut position = 0;
    while position < bytes.len() {
        let (byte, consumed) = decode_at(bytes, position);
        decoded.push(byte);
        position += consumed;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn step<'text>(mut words: Vec<&'text str>, word: &'text str) -> Vec<&'text str> {
    match word {
        "" | "." => {}
        ".." => drop(words.pop()),
        name => words.push(name),
    }
    words
}

#[must_use]
pub fn translate_path(root: &Path, request_path: &str) -> PathBuf {
    let decoded = percent_decode(request_path);
    let words = decoded.split('/').fold(Vec::new(), step);
    words
        .iter()
        .fold(root.to_path_buf(), |path, word| path.join(word))
}
