use std::time::Duration;

use crate::camera::VideoDeviceSettings;

const DEFAULT_RENDERER_PORT: &str = "3000";
const DEFAULT_DEVTOOLS_PORT: u16 = 9222;
const DEFAULT_VIDEO_DEVICE: &str = "/dev/video10";
const FRAME_PORT: u16 = 8768;
const OPEN_SETTLE: Duration = Duration::from_secs(3);
const MODAL_PAUSE: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub struct VirtualCameraSettings {
    pub devtools_port: u16,
    pub renderer_port: String,
    pub frame_port: u16,
    pub encoder_program: String,
    pub video: VideoDeviceSettings,
    pub open_settle: Duration,
    pub modal_pause: Duration,
}

fn text_of(lookup: &impl Fn(&str) -> Option<String>, name: &str, fallback: &str) -> String {
    lookup(name)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_owned())
}

impl VirtualCameraSettings {
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>, arguments: &[String]) -> Self {
        let device = text_of(&lookup, "V4L2_DEVICE", DEFAULT_VIDEO_DEVICE);
        let renderer_port = text_of(&lookup, "AVATAR_RENDERER_PORT", DEFAULT_RENDERER_PORT);
        let devtools_port = text_of(&lookup, "CDP_PORT", "9222");
        let video = video_settings(&device, arguments);
        Self::new(
            leading_integer(&devtools_port, DEFAULT_DEVTOOLS_PORT),
            renderer_port,
            video,
        )
    }

    fn new(devtools_port: u16, renderer_port: String, video: VideoDeviceSettings) -> Self {
        Self {
            devtools_port,
            renderer_port,
            frame_port: FRAME_PORT,
            encoder_program: "ffmpeg".to_owned(),
            video,
            open_settle: OPEN_SETTLE,
            modal_pause: MODAL_PAUSE,
        }
    }
}

fn video_settings(video_device: &str, arguments: &[String]) -> VideoDeviceSettings {
    VideoDeviceSettings {
        frames_per_second: numeric_argument(arguments, "--fps", 60),
        width: numeric_argument(arguments, "--width", 1280),
        height: numeric_argument(arguments, "--height", 720),
        video_device: video_device.to_owned(),
    }
}

fn numeric_argument(arguments: &[String], flag: &str, fallback: u32) -> u32 {
    arguments
        .windows(2)
        .find(|pair| pair.first().is_some_and(|name| name == flag))
        .and_then(|pair| pair.get(1))
        .map_or(fallback, |value| leading_integer(value, fallback))
}

fn leading_integer<Number: std::str::FromStr>(text: &str, fallback: Number) -> Number {
    let digits: String = text
        .trim_start()
        .trim_start_matches('+')
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().unwrap_or(fallback)
}
