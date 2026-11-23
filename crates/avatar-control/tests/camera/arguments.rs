use avatar_control::camera::VideoDeviceSettings;

fn settings(
    frames_per_second: u32,
    width: u32,
    height: u32,
    video_device: &str,
) -> VideoDeviceSettings {
    VideoDeviceSettings {
        frames_per_second,
        width,
        height,
        video_device: video_device.to_owned(),
    }
}

fn expected(video_size: &str, framerate: &str, device: &str) -> Vec<String> {
    [
        expected_input(video_size, framerate),
        expected_output(device),
    ]
    .concat()
}

const RAW_INPUT: [&str; 6] = [
    "-y",
    "-f",
    "rawvideo",
    "-pixel_format",
    "rgba",
    "-video_size",
];
const RATE: [&str; 1] = ["-framerate"];
const PIPE_INPUT: [&str; 2] = ["-i", "pipe:0"];
const DEVICE_OUTPUT: [&str; 8] = [
    "-vf",
    "vflip",
    "-pix_fmt",
    "yuv420p",
    "-threads:v",
    "1",
    "-f",
    "v4l2",
];

fn expected_input(video_size: &str, framerate: &str) -> Vec<String> {
    let parts: [&[&str]; 5] = [&RAW_INPUT, &[video_size], &RATE, &[framerate], &PIPE_INPUT];
    parts.concat().into_iter().map(str::to_owned).collect()
}

fn expected_output(device: &str) -> Vec<String> {
    [&DEVICE_OUTPUT[..], &[device]]
        .concat()
        .into_iter()
        .map(str::to_owned)
        .collect()
}

#[test]
fn default_capture_arguments_match_the_js_encoder_argv() {
    let arguments = settings(60, 1280, 720, "/dev/video10").encoder_arguments();
    assert_eq!(arguments, expected("1280x720", "60", "/dev/video10"));
}

#[test]
fn smaller_capture_uses_its_own_size_rate_and_device() {
    let arguments = settings(30, 640, 480, "/dev/video2").encoder_arguments();
    assert_eq!(arguments, expected("640x480", "30", "/dev/video2"));
}

#[test]
fn tiny_capture_keeps_the_device_path_as_the_final_argument() {
    let arguments = settings(1, 2, 2, "/tmp/x").encoder_arguments();
    assert_eq!(arguments, expected("2x2", "1", "/tmp/x"));
}
