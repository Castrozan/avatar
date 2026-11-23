use avatar_floor::quiet::Seat;
use avatar_listening::conversation_state::ListenerPurpose;
use serde_json::{Map, json};

use crate::talk::cli::arguments::OpenArguments;
use crate::talk::effects::bring_up_room;
use crate::talk::effects::{
    COMMAND_TIMEOUT_SECONDS, close_conversation, show_chosen_character, status,
};
use crate::talk::failure::TalkFailure;
use crate::talk::ports::exec::run_checked;
use crate::talk::ports::{AgentPane, Boundaries, Environment, ListeningRequest, Subprocess};
use crate::talk::state::addressee::current;
use crate::talk::state::seat::{microphone_source, seat_name};
use crate::talk::state::this_pane;
use crate::talk::surfaces::seat_preparation::prepare;

const HOST_VARIABLE: &str = "AVATAR_AGENT_HOST";
const LIVE_NEEDS_A_PANE: &str = "avatar-talk open --live must run from an agent's herdr pane: the live voice hands the room's tasks to that agent";
const LIVE_HAS_NO_DISCORD_SEAT: &str =
    "The live voice has no Discord seat; open a conversation instead";
const LIVE_NEEDS_A_SERVANT: &str = "The live voice could not give the floor to this pane: only a session with a Servant name can hold it and receive the room's tasks";

fn receiving_agent(environment: &impl Environment, open: &OpenArguments) -> AgentPane {
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

fn give_the_floor(boundary: &mut impl Boundaries, agent: &AgentPane) -> Result<(), TalkFailure> {
    let calling = AgentPane {
        host: None,
        pane: this_pane(boundary),
    };
    if *agent == calling {
        return claim_floor(boundary);
    }
    boundary.hand_to_remote_agent(agent).map(|_| ())
}

fn holds_the_floor(boundary: &mut impl Boundaries, agent: &AgentPane) -> Result<bool, TalkFailure> {
    let holder = current(boundary)?;
    Ok(holder.is_some_and(|held| held.host == agent.host && Some(held.pane) == agent.pane))
}

fn listen_for_the_live_voice(
    boundary: &mut impl Boundaries,
    open: &OpenArguments,
    seat: Seat,
) -> Result<(), TalkFailure> {
    bring_up_room(boundary, seat)?;
    prepare(boundary, seat)?;
    show_chosen_character(boundary, open)?;
    let mut settings = Map::new();
    settings.insert("seat".to_owned(), json!(seat_name(seat)));
    boundary.begin_listening(&ListeningRequest {
        purpose: ListenerPurpose::Live,
        human_source: microphone_source(seat).to_owned(),
        settings,
    })
}

fn seat_the_live_voice(
    boundary: &mut impl Boundaries,
    open: &OpenArguments,
    agent: &AgentPane,
    seat: Seat,
) -> Result<(), TalkFailure> {
    listen_for_the_live_voice(boundary, open, seat)?;
    give_the_floor(boundary, agent)?;
    let holds = holds_the_floor(boundary, agent)?;
    holds
        .then_some(())
        .ok_or_else(|| TalkFailure::new(LIVE_NEEDS_A_SERVANT))
}

fn live_seat(open: &OpenArguments, agent: &AgentPane) -> Result<Seat, TalkFailure> {
    if agent.pane.as_deref().is_none_or(str::is_empty) {
        return Err(TalkFailure::new(LIVE_NEEDS_A_PANE));
    }
    match open.seat.unwrap_or(Seat::Room) {
        Seat::Discord => Err(TalkFailure::new(LIVE_HAS_NO_DISCORD_SEAT)),
        seat => Ok(seat),
    }
}

pub fn open_live_conversation(
    boundary: &mut impl Boundaries,
    open: &OpenArguments,
) -> Result<(), TalkFailure> {
    let agent = receiving_agent(boundary, open);
    let seat = live_seat(open, &agent)?;
    if let Err(failure) = seat_the_live_voice(boundary, open, &agent, seat) {
        close_conversation(boundary)?;
        return Err(failure);
    }
    status(boundary)
}
