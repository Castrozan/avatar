use avatar_seat::holder::{SeatKeeper, TunnelProcess};
use serde_json::{Value, json};

use super::world::{World, next_of};

impl SeatKeeper for &World {
    fn look(&mut self, seat: Option<&Value>) -> bool {
        self.note(json!(["look", seat]));
        next_of(&mut self.queues.borrow_mut().keep, true)
    }

    fn end(&mut self) {
        self.note(json!(["end"]));
    }
}

impl TunnelProcess for &World {
    fn stays_open_for(&mut self, seconds: u64) -> bool {
        self.note(json!(["wait", seconds]));
        next_of(&mut self.queues.borrow_mut().rounds, "stays".to_owned()) != "ends"
    }

    fn exit_code(&mut self) -> Option<i32> {
        self.note(json!(["poll"]));
        self.spec.code
    }

    fn terminate(&mut self) {
        self.note(json!(["terminate"]));
    }
}
