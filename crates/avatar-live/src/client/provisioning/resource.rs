use serde_json::Value;

use super::api::AgentApi;
use super::containment::contains;
use crate::client::ClientError;

pub(super) struct Resource {
    pub(super) collection: &'static str,
    pub(super) listing: &'static str,
    pub(super) identifier: &'static str,
    pub(super) name_pointer: &'static str,
    pub(super) creation: &'static str,
    pub(super) name: &'static str,
    pub(super) desired: Value,
}

fn identifier_of(entry: &Value, resource: &Resource) -> Option<String> {
    entry[resource.identifier].as_str().map(str::to_owned)
}

fn named(listed: &Value, resource: &Resource) -> Option<String> {
    let entries = listed[resource.listing].as_array()?;
    let wanted = Value::from(resource.name);
    let mut matching = entries
        .iter()
        .filter(|entry| entry.pointer(resource.name_pointer) == Some(&wanted));
    matching.find_map(|entry| identifier_of(entry, resource))
}

async fn find(api: &AgentApi, resource: &Resource) -> Result<Option<String>, ClientError> {
    let search = format!(
        "{}?search={}&page_size=100",
        resource.collection, resource.name
    );
    Ok(named(&api.get(&search).await?, resource))
}

async fn update_if_different(
    api: &AgentApi,
    resource: &Resource,
    id: String,
) -> Result<String, ClientError> {
    let path = format!("{}/{id}", resource.collection);
    let current = api.get(&path).await?;
    if !contains(&current, &resource.desired) {
        api.patch(&path, &resource.desired).await?;
    }
    Ok(id)
}

async fn create(api: &AgentApi, resource: &Resource) -> Result<String, ClientError> {
    let created = api.post(resource.creation, &resource.desired).await?;
    identifier_of(&created, resource).ok_or_else(|| {
        ClientError(format!(
            "{} returned no {}",
            resource.creation, resource.identifier
        ))
    })
}

pub(super) async fn reconcile(api: &AgentApi, resource: &Resource) -> Result<String, ClientError> {
    match find(api, resource).await? {
        Some(id) => update_if_different(api, resource, id).await,
        None => create(api, resource).await,
    }
}
