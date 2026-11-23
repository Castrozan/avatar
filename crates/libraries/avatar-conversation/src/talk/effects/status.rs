use avatar_listening::conversation_state::ConversationPhase;

use super::report_lines::status_line;
use crate::talk::failure::TalkFailure;
use crate::talk::ports::{Boundaries, Files, Units};
use crate::talk::state::heard::last_heard;
use crate::talk::state::phase::phase;
use crate::talk::state::seat::{configured, seat_name};

fn open_seat(
    boundary: &mut (impl Units + Files),
    is_open: bool,
) -> Result<Option<&'static str>, TalkFailure> {
    is_open
        .then(|| configured(boundary).map(seat_name))
        .transpose()
}

pub fn status(boundary: &mut impl Boundaries) -> Result<(), TalkFailure> {
    let current = phase(boundary);
    let is_open = matches!(current, ConversationPhase::Open | ConversationPhase::Live);
    let seat = open_seat(boundary, is_open)?;
    let heard = last_heard(boundary)?;
    let live = current == ConversationPhase::Live;
    boundary.print_out(&format!("{}\n", status_line(is_open, live, seat, heard)));
    Ok(())
}
