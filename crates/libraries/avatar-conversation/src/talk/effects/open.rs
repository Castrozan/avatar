use avatar_floor::quiet::Seat;
use avatar_holder::default_holder::listener_settings;
use avatar_listening::conversation_state::ListenerPurpose;
use serde_json::{Map, Value, json};

use super::{close_conversation, show_chosen_character, status};
use crate::talk::cli::arguments::OpenArguments;
use crate::talk::failure::TalkFailure;
use crate::talk::ports::{Boundaries, ListeningRequest, Readiness, Units};
use crate::talk::state::CONVERSATION_TARGET;
use crate::talk::state::seat::{microphone_source, seat_name};
use crate::talk::surfaces::discord;
use crate::talk::surfaces::seat_preparation::prepare;

fn conversation_settings(open: &OpenArguments, seat: Seat) -> Map<String, Value> {
    let mut settings = Map::new();
    settings.insert("language".to_owned(), json!(open.language));
    settings.insert("barge_in".to_owned(), json!(true));
    settings.insert("seat".to_owned(), json!(seat_name(seat)));
    settings.extend(listener_settings(
        open.holder.as_deref(),
        open.character.as_deref(),
    ));
    settings
}

pub fn bring_up_room(units: &mut impl Units, seat: Seat) -> Result<(), TalkFailure> {
    let checks = [Readiness::ControlServer, Readiness::SeatMicrophone(seat)];
    units.bring_up(CONVERSATION_TARGET, &checks)
}

fn start_conversation(
    boundary: &mut impl Boundaries,
    open: &OpenArguments,
    seat: Seat,
) -> Result<(), TalkFailure> {
    bring_up_room(boundary, seat)?;
    prepare(boundary, seat)?;
    show_chosen_character(boundary, open)?;
    boundary.begin_listening(&ListeningRequest {
        purpose: ListenerPurpose::Conversation,
        human_source: microphone_source(seat).to_owned(),
        settings: conversation_settings(open, seat),
    })
}

pub fn open_conversation(
    boundary: &mut impl Boundaries,
    open: &OpenArguments,
) -> Result<(), TalkFailure> {
    let seat = open.seat.unwrap_or(Seat::Room);
    if let Err(failure) = start_conversation(boundary, open, seat) {
        abandon(boundary, seat)?;
        return Err(failure);
    }
    status(boundary)
}

fn abandon(boundary: &mut impl Boundaries, seat: Seat) -> Result<(), TalkFailure> {
    if seat == Seat::Discord {
        discord::leave(boundary);
    }
    close_conversation(boundary)
}

pub fn keep_conversation_open(
    boundary: &mut impl Boundaries,
    open: &OpenArguments,
) -> Result<(), TalkFailure> {
    show_chosen_character(boundary, open)?;
    status(boundary)
}
