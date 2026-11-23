use std::fs::File;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};

const WINDOW_CLASS: &str = "avatar-browser";
const CONTROLS_URL: &str = "http://localhost:3000/controls.html";
const FIXED_OPTIONS: [&str; 8] = [
    "--remote-debugging-port=0",
    "--autoplay-policy=no-user-gesture-required",
    "--use-fake-ui-for-media-stream",
    "--test-type",
    "--disable-background-timer-throttling",
    "--disable-renderer-backgrounding",
    "--disable-backgrounding-occluded-windows",
    "--ozone-platform=x11",
];

pub(super) fn browser_arguments(profile: &Path, url: &str) -> Vec<String> {
    let mut arguments = vec![
        format!("--user-data-dir={}", profile.display()),
        format!("--class={WINDOW_CLASS}"),
    ];
    arguments.extend(FIXED_OPTIONS.map(str::to_owned));
    arguments.push(format!("--app={url}"));
    arguments
}

pub(super) fn spawn_browser(program: &str, profile: &Path, url: &str) -> io::Result<Child> {
    let output = File::create(profile.join("browser.log"))?;
    Command::new(program)
        .args(browser_arguments(profile, url))
        .stdout(output.try_clone()?)
        .stderr(output)
        .process_group(0)
        .spawn()
}

pub(super) fn open_controls(program: &str, profile: &Path) -> io::Result<Child> {
    Command::new(program)
        .arg(format!("--user-data-dir={}", profile.display()))
        .arg(format!("--app={CONTROLS_URL}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}
