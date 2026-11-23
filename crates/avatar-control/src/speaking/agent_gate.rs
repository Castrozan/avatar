use std::fmt;

use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentChannelPlan {
    OpenAsIs,
    MutedUntilReleased,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentChannelError {
    NotRunning,
    MalformedStatus,
}

impl AgentChannelError {
    const fn message(self) -> &'static str {
        match self {
            Self::NotRunning => "Avatar audio is not running; run avatar-start",
            Self::MalformedStatus => "Avatar audio status is malformed",
        }
    }
}

impl fmt::Display for AgentChannelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for AgentChannelError {}

pub fn plan_from_status(status: &Value) -> Result<AgentChannelPlan, AgentChannelError> {
    if !is_truthy(status.get("running")) {
        return Err(AgentChannelError::NotRunning);
    }
    let agent = agent_channel(status)?;
    Ok(plan_for_mute(is_truthy(agent.get("mute"))))
}

fn agent_channel(status: &Value) -> Result<&Value, AgentChannelError> {
    status
        .get("channels")
        .and_then(|channels| channels.get("agent"))
        .ok_or(AgentChannelError::MalformedStatus)
}

fn plan_for_mute(muted: bool) -> AgentChannelPlan {
    if muted {
        return AgentChannelPlan::MutedUntilReleased;
    }
    AgentChannelPlan::OpenAsIs
}

fn is_truthy(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::Bool(flag)) => *flag,
        Some(Value::Number(number)) => number.as_f64().is_some_and(|magnitude| magnitude != 0.0),
        Some(Value::String(text)) => !text.is_empty(),
        Some(Value::Array(_) | Value::Object(_)) => true,
    }
}
