pub(super) fn encode_uri_component(text: &str) -> String {
    text.bytes().map(encode_byte).collect()
}

fn encode_byte(byte: u8) -> String {
    if is_unreserved(byte) {
        char::from(byte).to_string()
    } else {
        format!("%{byte:02X}")
    }
}

fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&byte)
}
