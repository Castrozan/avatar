use std::io;
use std::process::ExitCode;

use avatar_conversation::talk::surfaces::window::reveal_window;

use crate::support::runtime::session_directory;
use crate::talk::TalkBoundaries;

pub(super) fn run() -> io::Result<ExitCode> {
    let mut boundaries = TalkBoundaries::on_this_machine(session_directory());
    reveal_window(&mut boundaries)
        .map(|()| ExitCode::SUCCESS)
        .map_err(|failure| io::Error::other(failure.to_string()))
}
