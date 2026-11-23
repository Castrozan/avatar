use super::mock::RecordedRequest;

pub(crate) fn boundary_of(request: &RecordedRequest) -> String {
    let content_type = request
        .headers
        .iter()
        .find(|(name, _)| name == "content-type");
    content_type
        .and_then(|(_, value)| value.split("boundary=").nth(1))
        .unwrap_or_default()
        .to_owned()
}

pub(crate) fn replace_boundary_text(value: &str, boundary: &str) -> String {
    if boundary.is_empty() {
        return value.to_owned();
    }
    value.replace(boundary, "BOUNDARY")
}

pub(crate) fn replace_bytes(bytes: &[u8], boundary: &str) -> Vec<u8> {
    let needle = boundary.as_bytes();
    if needle.is_empty() {
        return bytes.to_vec();
    }
    let mut output = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        cursor += push_or_replace(&mut output, &bytes[cursor..], needle);
    }
    output
}

fn push_or_replace(output: &mut Vec<u8>, rest: &[u8], needle: &[u8]) -> usize {
    if rest.starts_with(needle) {
        output.extend_from_slice(b"BOUNDARY");
        return needle.len();
    }
    output.push(rest[0]);
    1
}
