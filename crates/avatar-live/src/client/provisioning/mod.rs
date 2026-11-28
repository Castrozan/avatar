mod api;
mod containment;
mod desired;
mod resource;

pub use api::AgentApi;
pub use containment::contains;
pub use desired::{
    AGENT_NAME, MAX_DURATION_SECONDS, SPEECH_MODEL, TURN_TIMEOUT_SECONDS, agent_settings,
};

use super::ClientError;
use desired::{agent_resource, tool_resource};
use resource::reconcile;

pub async fn provision(api: &AgentApi) -> Result<String, ClientError> {
    let tool_id = reconcile(api, &tool_resource()).await?;
    reconcile(api, &agent_resource(&tool_id)).await
}

pub async fn signed_url(api: &AgentApi, agent_id: &str) -> Result<String, ClientError> {
    let path = format!("/v1/convai/conversation/get-signed-url?agent_id={agent_id}");
    let signed = api.get(&path).await?;
    let url = signed["signed_url"].as_str().map(str::to_owned);
    url.ok_or_else(|| ClientError("the signed url response had no signed_url".to_owned()))
}
