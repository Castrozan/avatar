use std::fs;
use std::path::Path;

use avatar_conversation::talk::effects::report_lines::{
    closed_line, room_held_by_line, status_line,
};
use avatar_conversation::utterances::last_heard;
use avatar_listening::conversation_addressee::file_store::FileAddresseeStore;
use avatar_listening::conversation_addressee::store::AddresseeStore;
use avatar_listening::listener::session::HEARD_FILE;

use super::MacRoom;
use crate::session::Console;
use crate::talk::Reports;

fn heard_through(directory: &Path) -> u64 {
    let text = fs::read_to_string(directory.join(HEARD_FILE)).ok();
    text.and_then(|content| content.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

fn addressed_through(directory: &Path) -> Option<u64> {
    let current = FileAddresseeStore::new(directory).current().ok().flatten();
    current.and_then(|addressee| u64::try_from(addressee.heard_through).ok())
}

fn last_heard_in(directory: &Path) -> u64 {
    last_heard(heard_through(directory), addressed_through(directory))
}

impl<Talk> Reports for MacRoom<Talk> {
    fn print_status(&mut self, is_open: bool, live: bool, seat: Option<&str>) {
        let heard = last_heard_in(self.session.directory.path());
        self.print_line(&status_line(is_open, live, seat, heard));
    }

    fn print_room_held_by(&mut self, servant: &str) {
        self.print_line(&room_held_by_line(servant));
    }

    fn print_closed(&mut self) {
        self.print_line(&closed_line());
    }
}
