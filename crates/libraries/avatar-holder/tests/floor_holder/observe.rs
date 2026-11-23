use crate::room::SharedRoom;
use crate::voice::SharedVoice;
use avatar_holder::default_holder::DefaultHolder;

pub(crate) fn spoken(voice: &SharedVoice) -> Vec<String> {
    voice.0.borrow().spoken.clone()
}

pub(crate) fn reports(voice: &SharedVoice) -> Vec<String> {
    voice.0.borrow().reports.clone()
}

pub(crate) fn pauses(room: &SharedRoom) -> Vec<f64> {
    room.0.borrow().pauses.clone()
}

pub(crate) fn taken_floors(room: &SharedRoom) -> Vec<DefaultHolder> {
    room.0.borrow().taken_by.clone()
}

pub(crate) fn utterance_calls(room: &SharedRoom) -> usize {
    room.0.borrow().next_utterance_calls
}
