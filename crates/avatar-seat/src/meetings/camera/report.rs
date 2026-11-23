use super::layout::{CAMERA_ACTIVE, CAMERA_REPORT_WAIT_SECONDS};
use super::obs::ObsCamera;
use crate::local::POLL_SECONDS;

fn report_text(camera: &ObsCamera<'_>) -> Option<String> {
    let text = camera.store.read_text(&camera.config.report_path());
    text.filter(|report| !report.is_empty())
}

fn poll(camera: &ObsCamera<'_>) -> Option<String> {
    let report = report_text(camera);
    if report.is_none() {
        camera.clock.sleep(POLL_SECONDS);
    }
    report
}

fn wait_for_report(camera: &ObsCamera<'_>) -> Option<String> {
    let deadline = camera.clock.monotonic() + CAMERA_REPORT_WAIT_SECONDS;
    let mut report = None;
    while report.is_none() && camera.clock.monotonic() < deadline {
        report = poll(camera);
    }
    report
}

impl ObsCamera<'_> {
    #[must_use]
    pub fn started(&self) -> bool {
        report_text(self).is_some_and(|text| text == CAMERA_ACTIVE)
    }

    #[must_use]
    pub fn reported_camera_state(&self) -> Option<String> {
        wait_for_report(self)
    }
}
