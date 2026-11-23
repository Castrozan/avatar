use super::{CALL_DEVICE, MEETING_FLAG, MEETING_MICROPHONE};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatPrograms {
    pub python: String,
    pub seat: String,
    pub call_capture: String,
}

#[must_use]
pub fn capture_command(programs: &SeatPrograms, device: &str) -> Vec<String> {
    if device == CALL_DEVICE {
        return vec![programs.python.clone(), programs.call_capture.clone()];
    }
    let mut seat = vec![programs.python.clone(), programs.seat.clone()];
    if device == MEETING_MICROPHONE {
        seat.push(MEETING_FLAG.to_owned());
    }
    seat
}
