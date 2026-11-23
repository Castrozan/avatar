#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoDeviceSettings {
    pub frames_per_second: u32,
    pub width: u32,
    pub height: u32,
    pub video_device: String,
}

const RAW_INPUT_OPTIONS: &[&str] = &[
    "-y",
    "-f",
    "rawvideo",
    "-pixel_format",
    "rgba",
    "-video_size",
];
const RATE_OPTIONS: &[&str] = &["-framerate"];
const PIPE_INPUT_OPTIONS: &[&str] = &["-i", "pipe:0"];
const DEVICE_OUTPUT_OPTIONS: &[&str] = &[
    "-vf",
    "vflip",
    "-pix_fmt",
    "yuv420p",
    "-threads:v",
    "1",
    "-f",
    "v4l2",
];

impl VideoDeviceSettings {
    #[must_use]
    pub fn encoder_arguments(&self) -> Vec<String> {
        let mut arguments = input_arguments(self.width, self.height, self.frames_per_second);
        arguments.extend(output_arguments(&self.video_device));
        arguments
    }
}

fn input_arguments(width: u32, height: u32, frames_per_second: u32) -> Vec<String> {
    let mut arguments = words(RAW_INPUT_OPTIONS);
    arguments.push(format!("{width}x{height}"));
    arguments.extend(words(RATE_OPTIONS));
    arguments.push(frames_per_second.to_string());
    arguments.extend(words(PIPE_INPUT_OPTIONS));
    arguments
}

fn output_arguments(video_device: &str) -> Vec<String> {
    let mut arguments = words(DEVICE_OUTPUT_OPTIONS);
    arguments.push(video_device.to_owned());
    arguments
}

fn words(options: &[&str]) -> Vec<String> {
    options.iter().map(|word| (*word).to_owned()).collect()
}
