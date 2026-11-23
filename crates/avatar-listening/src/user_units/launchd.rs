pub const CONVERSATION_AGENT: &str = "com.dotfiles.avatar-conversation";
pub const LIVE_AGENT: &str = "com.dotfiles.avatar-live";
const CONVERSATION_TARGET: &str = "avatar-conversation.target";
const RUNNING_LINE: &str = "\tstate = running";

pub trait LaunchctlPrint {
    fn printed(&mut self, agent_target: &str) -> String;
}

#[must_use]
fn agents_for(unit: &str) -> &'static [&'static str] {
    match unit {
        CONVERSATION_TARGET => &[CONVERSATION_AGENT, LIVE_AGENT],
        "avatar-listener.service" => &[CONVERSATION_AGENT],
        "avatar-live.service" => &[LIVE_AGENT],
        "avatar-control-server.service" => &["com.dotfiles.avatar-control"],
        "avatar-renderer.service" => &["com.dotfiles.avatar-renderer"],
        _ => &[],
    }
}

#[must_use]
fn agent_target(user_identifier: u32, agent: &str) -> String {
    format!("gui/{user_identifier}/{agent}")
}

#[must_use]
fn reports_running(printed: &str) -> bool {
    printed.split('\n').any(|line| line == RUNNING_LINE)
}

#[must_use]
pub fn unit_active(launchctl: &mut impl LaunchctlPrint, user_identifier: u32, unit: &str) -> bool {
    agents_for(unit).iter().any(|agent| {
        let printed = launchctl.printed(&agent_target(user_identifier, agent));
        reports_running(&printed)
    })
}
