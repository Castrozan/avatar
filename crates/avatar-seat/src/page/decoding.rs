fn hex_byte(pair: &[u8]) -> Option<u8> {
    if !pair.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let text = std::str::from_utf8(pair).ok()?;
    u8::from_str_radix(text, 16).ok()
}

fn escape_at(bytes: &[u8], index: usize) -> Option<u8> {
    let escaped = bytes.get(index + 1..index + 3).and_then(hex_byte);
    escaped.filter(|_| bytes.get(index) == Some(&b'%'))
}

pub(crate) fn percent_decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        let escaped = escape_at(bytes, index);
        decoded.push(escaped.unwrap_or(byte));
        index += escaped.map_or(1, |_| 3);
    }
    String::from_utf8_lossy(&decoded).into_owned()
}
