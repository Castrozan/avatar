use crate::talk::cli::arguments::OpenArguments;
use crate::talk::effects::COMMAND_TIMEOUT_SECONDS;
use crate::talk::failure::TalkFailure;
use crate::talk::ports::exec::run_checked;
use crate::talk::ports::{AgentPane, Boundaries, Environment, Subprocess};
use crate::talk::state::addressee::current;
use crate::talk::state::this_pane;

const HOST_VARIABLE: &str = "AVATAR_AGENT_HOST";

pub(super) fn receiving_agent(environment: &impl Environment, open: &OpenArguments) -> AgentPane {
    let this_machine = environment.variable(HOST_VARIABLE);
    open.agent.as_ref().map_or_else(
        || AgentPane {
            host: None,
            pane: this_pane(environment),
        },
        |agent| agent.resolve(this_machine.as_deref()),
    )
}

fn claim_floor(process: &mut impl Subprocess) -> Result<(), TalkFailure> {
    let argv = ["avatar-claim-floor"];
    run_checked(process, &argv, Some(COMMAND_TIMEOUT_SECONDS)).map(|_| ())
}

pub(super) fn give_the_floor(
    boundary: &mut impl Boundaries,
    agent: &AgentPane,
) -> Result<(), TalkFailure> {
    let calling = AgentPane {
        host: None,
        pane: this_pane(boundary),
    };
    if *agent == calling {
        return claim_floor(boundary);
    }
    boundary.hand_to_remote_agent(agent).map(|_| ())
}

pub(super) fn holds_the_floor(
    boundary: &mut impl Boundaries,
    agent: &AgentPane,
) -> Result<bool, TalkFailure> {
    let holder = current(boundary)?;
    Ok(holder.is_some_and(|held| held.host == agent.host && Some(held.pane) == agent.pane))
}
