pub const PANE_VARIABLE: &str = "HERDR_PANE_ID";
pub const CALLER_HOLDS_INPUT_VARIABLE: &str = "AVATAR_CALLER_HOLDS_INPUT";
const TALK_COMMAND: &str = "avatar-talk";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForwardedCommand {
    pub room: String,
    pub command: Vec<String>,
}

#[must_use]
pub fn this_host(configured_host: Option<&str>, hostname: &str) -> String {
    configured_host
        .filter(|host| !host.is_empty())
        .unwrap_or_else(|| {
            hostname
                .split_once('.')
                .map_or(hostname, |(short, _)| short)
        })
        .to_owned()
}

#[must_use]
pub fn command_in_the_room(arguments: &[String], pane: Option<&str>) -> Vec<String> {
    let mut command = Vec::new();
    if let Some(identity) = pane.filter(|value| !value.is_empty()) {
        command.push("env".to_owned());
        command.push(format!("{PANE_VARIABLE}={identity}"));
    }
    command.push(TALK_COMMAND.to_owned());
    command.extend(arguments.iter().cloned());
    command
}

#[must_use]
pub fn forwarding_plan(
    room: Option<&str>,
    this_host: &str,
    arguments: &[String],
    pane: Option<&str>,
) -> Option<ForwardedCommand> {
    let room = room.filter(|value| !value.is_empty() && *value != this_host)?;
    Some(ForwardedCommand {
        room: room.to_owned(),
        command: command_in_the_room(arguments, pane),
    })
}
