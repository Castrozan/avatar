use serde_json::Value;

use super::LOOKS_WITHOUT_A_MEETING_BEFORE_IT_ENDED;
use super::hold::SeatKeeper;
use crate::support::truth::python_truthy;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraFailure(pub String);

pub trait MeetingCamera {
    fn started(&mut self) -> Result<bool, CameraFailure>;
    fn running(&mut self) -> Result<bool, CameraFailure>;
    fn start(&mut self) -> Result<(), CameraFailure>;
    fn stop(&mut self) -> Result<(), CameraFailure>;
}

pub trait Reporter {
    fn report(&self, message: &str);
}

pub struct ConversationSeat;

impl SeatKeeper for ConversationSeat {
    fn look(&mut self, _seat: Option<&Value>) -> bool {
        true
    }

    fn end(&mut self) {}
}

pub struct MeetingSeat<'edges, Camera: MeetingCamera> {
    pub camera: Camera,
    pub owed: bool,
    pub looks_without_a_meeting: u32,
    reporter: &'edges dyn Reporter,
}

fn without_failing(reporter: &dyn Reporter, outcome: Result<(), CameraFailure>) {
    if let Err(CameraFailure(message)) = outcome {
        reporter.report(&message);
    }
}

fn revive(camera: &mut impl MeetingCamera, reporter: &dyn Reporter) -> Result<(), CameraFailure> {
    let started = camera.start();
    if started.is_err() {
        without_failing(reporter, camera.stop());
    }
    started
}

fn keep_the_camera(seat: &mut MeetingSeat<'_, impl MeetingCamera>) -> Result<(), CameraFailure> {
    seat.owed = seat.owed || seat.camera.started()?;
    if seat.owed && !seat.camera.running()? {
        return revive(&mut seat.camera, seat.reporter);
    }
    Ok(())
}

fn has_meeting(seat: &Value) -> bool {
    seat.get("meeting").is_some_and(python_truthy)
}

fn count_absence(seat: &mut MeetingSeat<'_, impl MeetingCamera>) -> bool {
    seat.looks_without_a_meeting += 1;
    seat.looks_without_a_meeting < LOOKS_WITHOUT_A_MEETING_BEFORE_IT_ENDED
}

fn look_at(seat: &mut MeetingSeat<'_, impl MeetingCamera>, found: &Value) -> bool {
    if !has_meeting(found) {
        return count_absence(seat);
    }
    seat.looks_without_a_meeting = 0;
    let kept = keep_the_camera(seat);
    without_failing(seat.reporter, kept);
    true
}

impl<'edges, Camera: MeetingCamera> MeetingSeat<'edges, Camera> {
    pub fn new(camera: Camera, reporter: &'edges dyn Reporter) -> Self {
        Self {
            camera,
            owed: false,
            looks_without_a_meeting: 0,
            reporter,
        }
    }
}

impl<Camera: MeetingCamera> SeatKeeper for MeetingSeat<'_, Camera> {
    fn look(&mut self, seat: Option<&Value>) -> bool {
        seat.is_none_or(|found| look_at(self, found))
    }

    fn end(&mut self) {
        self.owed = false;
        without_failing(self.reporter, self.camera.stop());
    }
}
