use std::cell::RefCell;

use avatar_floor::quiet::Seat;
use avatar_seat::edges::EdgeFailure;
use avatar_seat::local::SeatFailure;
use avatar_seat::meetings::readiness::Readiness;
use avatar_seat::meetings::start::SeatPreparation;
use serde_json::{Value, json};

use crate::fakes::processes::Log;

pub(crate) struct Queue(RefCell<Vec<bool>>);

impl Queue {
    pub(crate) fn from_value(values: &Value) -> Self {
        let flags = values.as_array().cloned().unwrap_or_default();
        Self(RefCell::new(
            flags.iter().map(|flag| flag == true).collect(),
        ))
    }

    fn pop(&self) -> bool {
        let mut flags = self.0.borrow_mut();
        let first = flags.first().copied().unwrap_or(true);
        let removable = usize::from(flags.len() > 1);
        drop(flags.drain(..removable));
        first
    }
}

pub(crate) struct ScriptedReadiness {
    pub(crate) audio: Queue,
    pub(crate) renderer: Queue,
    pub(crate) camera: Queue,
    pub(crate) log: Log,
}

impl ScriptedReadiness {
    fn answer(&self, mark: &str, queue: &Queue) -> bool {
        self.log.borrow_mut().push(json!(mark));
        queue.pop()
    }
}

impl Readiness for ScriptedReadiness {
    fn audio_ready(&self) -> Result<bool, EdgeFailure> {
        Ok(self.answer("A", &self.audio))
    }

    fn renderer_ready(&self) -> Result<bool, EdgeFailure> {
        Ok(self.answer("R", &self.renderer))
    }

    fn camera_ready(&self) -> Result<bool, EdgeFailure> {
        Ok(self.answer("C", &self.camera))
    }
}

pub(crate) struct ScriptedSeats {
    pub(crate) failure: Option<String>,
    pub(crate) log: Log,
}

fn seat_name(seat: Seat) -> &'static str {
    match seat {
        Seat::Remote => "remote",
        _ => "room",
    }
}

impl SeatPreparation for ScriptedSeats {
    fn prepare(&self, seat: Seat) -> Result<(), SeatFailure> {
        self.log
            .borrow_mut()
            .push(json!({"kind": "prepare", "seat": seat_name(seat)}));
        self.failure
            .clone()
            .map_or(Ok(()), |message| Err(SeatFailure::Timeout(message)))
    }
}
