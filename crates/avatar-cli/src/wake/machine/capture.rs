use std::path::Path;

use avatar_conversation::talk::state::phase::live_purpose;
use avatar_listening::conversation_state::ListenerPurpose;
use avatar_listening::listener::runtime::listening;
use avatar_listening::listener::units::SystemdUnits;

use crate::talk::TalkBoundaries;

pub(crate) trait LocalCapture {
    fn listening(&self) -> bool;
    fn live_purpose(&self, directory: &Path) -> Option<ListenerPurpose>;
}

pub(super) struct SystemdCapture;

impl LocalCapture for SystemdCapture {
    fn listening(&self) -> bool {
        listening(&mut SystemdUnits::new("systemctl"))
    }

    fn live_purpose(&self, directory: &Path) -> Option<ListenerPurpose> {
        live_purpose(&mut TalkBoundaries::on_this_machine(
            directory.to_path_buf(),
        ))
    }
}
