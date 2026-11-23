const TRAILING_FLAGS: [&str; 7] = [
    "--create-if-missing",
    "--reasoning",
    "low",
    "--source",
    "tool",
    "--format",
    "stream-json",
];

#[must_use]
pub fn hermes_command(profile: &str, prompt: &str, day: &str) -> Vec<String> {
    let mut command = leading_arguments(profile, prompt);
    command.push(format!("voice-call-{day}"));
    command.extend(TRAILING_FLAGS.map(str::to_owned));
    command
}

fn leading_arguments(profile: &str, prompt: &str) -> Vec<String> {
    let words = ["hermes", "-p", profile, "chat", "--query", prompt];
    words
        .into_iter()
        .chain(["--oneshot", "--continue"])
        .map(str::to_owned)
        .collect()
}
