use super::boundaries::ServantCharacter;
use super::session::Session;
use crate::addressee::{Addressee, current, hand_to};
use std::io;
use std::path::Path;

#[must_use]
pub fn addressee_of(session: &Session, heard_through: i64) -> Addressee {
    Addressee {
        pane: session.pane.clone(),
        servant: session.servant.clone(),
        heard_through,
        host: session.host.clone(),
    }
}

pub fn give_the_room_to(
    directory: &Path,
    addressee: &Addressee,
    character: &dyn ServantCharacter,
) -> io::Result<()> {
    let previous = current(directory)?;
    hand_to(directory, addressee)?;
    if previous.is_none_or(|held| held.servant != addressee.servant) {
        character.show_for(&addressee.servant);
    }
    Ok(())
}
