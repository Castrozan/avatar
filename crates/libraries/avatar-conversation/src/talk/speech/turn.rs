use crate::talk::effects::report_lines::withheld_line;
use crate::talk::failure::TalkFailure;
use crate::talk::hearing::{POLL_SECONDS, read_since};
use crate::talk::ports::{Clock, Files};
use crate::talk::state::heard::last_heard;
use crate::turn_taking::{WithheldReason, queued_behind_a_stopped_reply};

const STOPPED_REPLY_FILE: &str = "stopped-reply";
const SPEECH_IN_PROGRESS_MARKER: &str = "speech-in-progress";
const QUIET_WAIT_SECONDS: f64 = 3.0;

fn queued(boundary: &mut (impl Files + Clock)) -> bool {
    let modified = boundary.modified_at(STOPPED_REPLY_FILE);
    let age = modified.map(|at| boundary.wall() - at);
    queued_behind_a_stopped_reply(age)
}

fn keep_waiting(boundary: &mut (impl Files + Clock), deadline: f64) -> bool {
    let waiting = boundary.monotonic() < deadline;
    if waiting {
        boundary.sleep(POLL_SECONDS);
    }
    waiting
}

fn quiet_within(boundary: &mut (impl Files + Clock), seconds: f64) -> bool {
    let deadline = boundary.monotonic() + seconds;
    let mut present = boundary.exists(SPEECH_IN_PROGRESS_MARKER);
    while present && keep_waiting(boundary, deadline) {
        present = boundary.exists(SPEECH_IN_PROGRESS_MARKER);
    }
    !present
}

fn unheard_words(files: &mut impl Files) -> Result<bool, TalkFailure> {
    let heard = last_heard(files)?;
    Ok(!read_since(files, heard)?.is_empty())
}

pub(super) fn withheld_reason(
    boundary: &mut (impl Files + Clock),
) -> Result<Option<WithheldReason>, TalkFailure> {
    if queued(boundary) {
        return Ok(Some(WithheldReason::Queued));
    }
    if !quiet_within(boundary, QUIET_WAIT_SECONDS) {
        return Ok(Some(WithheldReason::Speaking));
    }
    Ok(unheard_words(boundary)?.then_some(WithheldReason::Unheard))
}

pub(super) fn remember_stopped_reply(files: &mut impl Files) {
    files.touch(STOPPED_REPLY_FILE);
}

pub(super) fn withhold(files: &mut impl Files, reason: WithheldReason) -> String {
    remember_stopped_reply(files);
    withheld_line(reason)
}
