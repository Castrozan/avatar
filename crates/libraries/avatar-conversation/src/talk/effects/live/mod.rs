use avatar_floor::quiet::Seat;
use avatar_listening::conversation_state::ListenerPurpose;
use serde_json::{Map, json};

mod floor;
mod stage;

use floor::{give_the_floor, holds_the_floor, receiving_agent};
pub use stage::{LiveStage, RoomStage};

use crate::talk::cli::arguments::OpenArguments;
use crate::talk::effects::{show_chosen_character, status};
use crate::talk::failure::TalkFailure;
use crate::talk::ports::{AgentPane, Boundaries, ListeningRequest};
use crate::talk::state::seat::{microphone_source, seat_name};
use crate::talk::surfaces::seat_preparation::prepare;

const LIVE_NEEDS_A_PANE: &str = "avatar-talk open --live must run from an agent's herdr pane: the live voice hands the room's tasks to that agent";
const LIVE_HAS_NO_DISCORD_SEAT: &str =
    "The live voice has no Discord seat; open a conversation instead";
const LIVE_NEEDS_A_SERVANT: &str = "The live voice could not give the floor to this pane: only a session with a Servant name can hold it and receive the room's tasks";

fn listen_for_the_live_voice<Boundary: Boundaries>(
    boundary: &mut Boundary,
    open: &OpenArguments,
    seat: Seat,
    stage: &mut impl LiveStage<Boundary>,
) -> Result<(), TalkFailure> {
    stage.bring_up(boundary, seat)?;
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

fn seat_the_live_voice<Boundary: Boundaries>(
    boundary: &mut Boundary,
    request: (&OpenArguments, &AgentPane, Seat),
    stage: &mut impl LiveStage<Boundary>,
) -> Result<(), TalkFailure> {
    let (open, agent, seat) = request;
    listen_for_the_live_voice(boundary, open, seat, stage)?;
    stage.wait_until_seated(boundary)?;
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
    open_live_conversation_on(boundary, open, &mut RoomStage)
}

pub fn open_live_conversation_on<Boundary: Boundaries>(
    boundary: &mut Boundary,
    open: &OpenArguments,
    stage: &mut impl LiveStage<Boundary>,
) -> Result<(), TalkFailure> {
    let agent = receiving_agent(boundary, open);
    let seat = live_seat(open, &agent)?;
    if let Err(failure) = seat_the_live_voice(boundary, (open, &agent, seat), stage) {
        stage.close(boundary)?;
        return Err(failure);
    }
    status(boundary)
}
