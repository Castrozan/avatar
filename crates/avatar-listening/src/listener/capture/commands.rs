pub const DISCORD_DEVICE: &str = "discord";
pub const DISCORD_CAPTURE_COMMAND: [&str; 2] = ["discord-voice", "capture"];
const PULSE_OPTIONS: [&str; 8] = [
    "--client-name=avatar-listener",
    "--property=node.dont-reconnect=true",
    "--property=node.dont-fallback=true",
    "--raw",
    "--format=s16le",
    "--rate=16000",
    "--channels=1",
    "--latency-msec=10",
];

#[must_use]
pub fn capture_command(device: &str) -> Vec<String> {
    let mut words = vec!["parec".to_owned(), format!("--device={device}")];
    words.extend(PULSE_OPTIONS.map(str::to_owned));
    words
}

fn discord_command() -> Vec<String> {
    DISCORD_CAPTURE_COMMAND.map(str::to_owned).to_vec()
}

pub fn command_by_device(
    machine_command: impl Fn(&str) -> Vec<String>,
) -> impl Fn(&str) -> Vec<String> {
    move |device| match device {
        DISCORD_DEVICE => discord_command(),
        other => machine_command(other),
    }
}
