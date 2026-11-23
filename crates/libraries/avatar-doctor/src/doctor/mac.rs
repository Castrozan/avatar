use super::command::CommandRunner;
use super::failure::DoctorError;
use super::finding::Finding;

const WAKE_AGENT: &str = "com.dotfiles.avatar-wake";
const CONVERSATION_AGENT: &str = "com.dotfiles.avatar-conversation";

pub(crate) fn mac_agents_loaded(
    runner: &impl CommandRunner,
    user_id: u32,
) -> Result<Vec<Finding>, DoctorError> {
    Ok(vec![
        agent_loaded(runner, "wake agent", WAKE_AGENT, user_id)?,
        agent_loaded(runner, "conversation agent", CONVERSATION_AGENT, user_id)?,
    ])
}

fn agent_loaded(
    runner: &impl CommandRunner,
    name: &str,
    label: &str,
    user_id: u32,
) -> Result<Finding, DoctorError> {
    let target = format!("gui/{user_id}/{label}");
    let loaded = runner.run(&["launchctl", "print", &target], &[])?.succeeded;
    let detail = if loaded { "loaded" } else { "not loaded" };
    Ok(Finding::new(name, loaded, detail))
}
