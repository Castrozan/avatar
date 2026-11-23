use super::super::layout::{FRAME_HEIGHT, FRAME_WIDTH, FRAMES_PER_SECOND, PROFILE_NAME};

const VIDEO_SETTINGS: [(&str, u32); 6] = [
    ("BaseCX", FRAME_WIDTH),
    ("BaseCY", FRAME_HEIGHT),
    ("OutputCX", FRAME_WIDTH),
    ("OutputCY", FRAME_HEIGHT),
    ("FPSType", 0),
    ("FPSCommon", FRAMES_PER_SECOND),
];

#[must_use]
pub fn profile_settings() -> String {
    let lines: Vec<String> = (VIDEO_SETTINGS.iter())
        .map(|(key, value)| format!("{key}={value}\n"))
        .collect();
    let video = lines.concat();
    format!("[General]\nName={PROFILE_NAME}\n\n[Video]\n{video}")
}
