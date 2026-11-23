use avatar_floor::addressee::Addressee;
use avatar_floor::floor::{RemoteAgent, may_speak};

pub const NO_SERVANT_MESSAGE: &str =
    "The pane has no Servant name: only a session with one can hold the avatar's floor.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClaimOutcome {
    Refused { message: String },
    Speaks { servant: Option<String> },
}

#[must_use]
pub fn refusal_message(holder_servant: &str) -> String {
    format!("{holder_servant} holds the avatar's floor: stay silent until the room names you.")
}

#[must_use]
pub fn pane_claim_outcome(
    holder: Option<&Addressee>,
    pane: &str,
    live_servant: impl FnOnce() -> Option<String>,
) -> ClaimOutcome {
    match holder.filter(|held| !may_speak(Some(held), pane)) {
        Some(held) => refused_by(held),
        None => ClaimOutcome::Speaks {
            servant: holder.map_or_else(live_servant, |held| Some(held.servant.clone())),
        },
    }
}

#[must_use]
pub fn speaker_claim_outcome(
    holder: Option<&Addressee>,
    speaker: &RemoteAgent,
    servant: Option<String>,
) -> ClaimOutcome {
    match holder.filter(|held| !same_pane_and_host(held, speaker)) {
        Some(held) => refused_by(held),
        None => ClaimOutcome::Speaks { servant },
    }
}

fn refused_by(holder: &Addressee) -> ClaimOutcome {
    ClaimOutcome::Refused {
        message: refusal_message(&holder.servant),
    }
}

fn same_pane_and_host(held: &Addressee, speaker: &RemoteAgent) -> bool {
    held.pane == speaker.pane && held.host == speaker.host
}
