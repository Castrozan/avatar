use std::sync::PoisonError;
use std::time::Duration;

use avatar_room_audio::mouth::{MouthLevels, VoiceLevelSink};

use super::shared::{Shared, fail, held, now_of, running};

const MOUTH_POLL_SECONDS: f64 = 0.25;

pub(super) fn mouth_wait<Voice: VoiceLevelSink>(
    shared: &Shared<Voice>,
    levels: &MouthLevels<Voice>,
) -> Duration {
    let wait = levels.seconds_until_next_delivery(now_of(shared));
    Duration::from_secs_f64(wait.unwrap_or(MOUTH_POLL_SECONDS))
}

pub(super) fn mouth_step<Voice: VoiceLevelSink>(shared: &Shared<Voice>) -> std::io::Result<()> {
    let levels = held(&shared.mouth.levels);
    let wait = mouth_wait(shared, &levels);
    let waited = shared.mouth.arrived.wait_timeout(levels, wait);
    waited
        .unwrap_or_else(PoisonError::into_inner)
        .0
        .deliver_due(now_of(shared))
}

pub(super) fn deliver_mouth<Voice: VoiceLevelSink>(shared: &Shared<Voice>) {
    while running(shared)
        && mouth_step(shared)
            .map_err(|error| fail(shared, &error.to_string()))
            .is_ok()
    {}
}
