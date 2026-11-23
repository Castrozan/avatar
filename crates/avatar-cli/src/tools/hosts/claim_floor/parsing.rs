use avatar_floor::floor::RemoteAgent;

const HOST_VARIABLE: &str = "AVATAR_AGENT_HOST";

fn valid_host(host: &str) -> bool {
    let mut characters = host.chars();
    let starts = characters
        .next()
        .is_some_and(|first| first.is_ascii_alphanumeric());
    starts && characters.all(|rest| rest.is_ascii_alphanumeric() || rest == '.' || rest == '-')
}

pub(super) fn parse_remote_agent(text: &str) -> Result<RemoteAgent, String> {
    let (host, pane) = text.split_once(':').unwrap_or((text, ""));
    if !valid_host(host) || pane.is_empty() {
        return Err(format!("{text:?} is not HOST:PANE"));
    }
    let this_machine = std::env::var(HOST_VARIABLE).ok();
    Ok(RemoteAgent {
        pane: pane.to_owned(),
        host: (this_machine.as_deref() != Some(host)).then(|| host.to_owned()),
    })
}
