pub const SCRIBE_MODEL: &str = "scribe_v2";

#[must_use]
pub fn scribe_fields(language: Option<&str>, keyterms: &[String]) -> Vec<(String, String)> {
    let mut fields = vec![
        field("model_id", SCRIBE_MODEL),
        field("tag_audio_events", "false"),
    ];
    if let Some(code) = language.filter(|code| !code.is_empty()) {
        fields.push(field("language_code", code));
    }
    fields.extend(keyterms.iter().map(|term| field("keyterms", term)));
    fields
}

fn field(name: &str, value: &str) -> (String, String) {
    (name.to_owned(), value.to_owned())
}

#[must_use]
pub fn multipart_body(boundary: &str, fields: &[(String, String)], wav: &[u8]) -> Vec<u8> {
    let mut body: Vec<u8> = fields
        .iter()
        .flat_map(|(name, value)| field_part(boundary, name, value))
        .collect();
    body.extend(file_part(boundary));
    body.extend_from_slice(wav);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

fn field_part(boundary: &str, name: &str, value: &str) -> Vec<u8> {
    format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n")
        .into_bytes()
}

fn file_part(boundary: &str) -> Vec<u8> {
    format!("--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"segment.wav\"\r\nContent-Type: audio/wav\r\n\r\n").into_bytes()
}
