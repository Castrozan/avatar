use avatar_holder::default_holder::{self, DefaultHolder, configured_holder};
use avatar_listening::conversation_addressee::Addressee;

use crate::talk::failure::TalkFailure;
use crate::talk::ports::exec::run_checked;
use crate::talk::ports::{Boundaries, Files, Subprocess};
use crate::talk::state::addressee::current;
use crate::talk::state::configuration::read_configuration;
use crate::talk::state::heard::last_heard;

const SWITCH_TIMEOUT_SECONDS: f64 = 30.0;

fn configured(files: &mut impl Files) -> Result<Option<DefaultHolder>, TalkFailure> {
    let configuration = read_configuration(files)?;
    configured_holder(&configuration).map_err(|failure| TalkFailure::new(failure.to_string()))
}

pub fn floor_returns_to(
    files: &mut impl Files,
    pane: Option<&str>,
) -> Result<Option<DefaultHolder>, TalkFailure> {
    let holder = configured(files)?;
    let holding = current(files)?.map(|addressee| addressee.pane);
    let returning = pane.and_then(|asked| {
        default_holder::floor_returns_to(holder.as_ref(), holding.as_deref(), asked)
    });
    Ok(returning.cloned())
}

pub fn show_character(process: &mut impl Subprocess, character: Option<&str>) {
    let Some(name) = character else {
        return;
    };
    let _ = run_checked(
        process,
        &["avatar-command", "character", name],
        Some(SWITCH_TIMEOUT_SECONDS),
    );
}

pub fn take_the_floor(
    boundary: &mut impl Boundaries,
    holder: &DefaultHolder,
) -> Result<(), TalkFailure> {
    let heard_through = i64::try_from(last_heard(boundary)?).unwrap_or(i64::MAX);
    boundary.give_the_room_to(&Addressee {
        pane: holder.pane(),
        servant: holder.servant(),
        heard_through,
        host: None,
    })?;
    show_character(boundary, holder.character());
    Ok(())
}
