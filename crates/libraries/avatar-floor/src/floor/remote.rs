use super::boundaries::ServantCharacter;
use super::room::give_the_room_to;
use crate::addressee::Addressee;
use std::io;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteAgent {
    pub pane: String,
    pub host: Option<String>,
}

pub fn hand_to_remote_agent(
    directory: &Path,
    agent: &RemoteAgent,
    heard_through: i64,
    servant_at: &dyn Fn(&RemoteAgent) -> Option<String>,
    character: &dyn ServantCharacter,
) -> io::Result<bool> {
    let Some(servant) = servant_at(agent).filter(|servant| !servant.is_empty()) else {
        return Ok(false);
    };
    let addressee = addressee_of_remote_agent(agent, servant, heard_through);
    give_the_room_to(directory, &addressee, character)?;
    Ok(true)
}

fn addressee_of_remote_agent(
    agent: &RemoteAgent,
    servant: String,
    heard_through: i64,
) -> Addressee {
    Addressee {
        pane: agent.pane.clone(),
        servant,
        heard_through,
        host: agent.host.clone(),
    }
}
