use std::path::Path;

const TABLE: [(&str, &str); 11] = [
    ("wav", "audio/wav"),
    ("mp3", "audio/mpeg"),
    ("ogg", "audio/ogg"),
    ("oga", "audio/ogg"),
    ("flac", "audio/x-flac"),
    ("m4a", "audio/mp4"),
    ("mp4", "audio/mp4"),
    ("aac", "audio/aac"),
    ("webm", "audio/webm"),
    ("json", "application/json; charset=UTF-8"),
    ("txt", "text/plain; charset=UTF-8"),
];

pub(super) fn for_path(path: &Path) -> &'static str {
    let extension = path
        .extension()
        .map(|text| text.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    TABLE
        .iter()
        .find(|(known, _)| *known == extension)
        .map_or("application/octet-stream", |(_, kind)| kind)
}
