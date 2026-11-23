use std::path::Path;

const TYPES: [(&str, &str); 30] = [
    ("js", "text/javascript"),
    ("mjs", "text/javascript"),
    ("css", "text/css"),
    ("json", "application/json"),
    ("png", "image/png"),
    ("txt", "text/plain"),
    ("wasm", "application/wasm"),
    ("svg", "image/svg+xml"),
    ("glb", "model/gltf-binary"),
    ("woff2", "font/woff2"),
    ("ico", "image/vnd.microsoft.icon"),
    ("html", "text/html"),
    ("htm", "text/html"),
    ("mp3", "audio/mpeg"),
    ("wav", "audio/x-wav"),
    ("ogg", "audio/ogg"),
    ("webp", "image/webp"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("xml", "text/xml"),
    ("pdf", "application/pdf"),
    ("gif", "image/gif"),
    ("webm", "video/webm"),
    ("mp4", "video/mp4"),
    ("gz", "application/gzip"),
    ("csv", "text/csv"),
    ("ttf", "font/ttf"),
    ("otf", "font/otf"),
    ("bmp", "image/bmp"),
    ("zip", "application/zip"),
];

#[must_use]
pub fn guess_type(path: &Path) -> &'static str {
    let extension = path
        .extension()
        .map(|text| text.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    TYPES
        .iter()
        .find(|(known, _)| *known == extension)
        .map_or("application/octet-stream", |(_, kind)| kind)
}
