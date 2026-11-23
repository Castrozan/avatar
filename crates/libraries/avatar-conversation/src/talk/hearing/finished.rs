use super::watch::ListeningWatch;
use super::{POLL_SECONDS, read_since};
use crate::talk::failure::TalkFailure;
use crate::talk::ports::{Clock, Files, Units};
use crate::talk::state::heard::last_heard;
use crate::utterances::{HeardEntry, SpeechSettling, ThrottledCheck};

const OPEN_CHECK_SECONDS: f64 = 1.0;
const UNFINISHED_SPEECH_LIMIT_SECONDS: f64 = 120.0;
const SPEECH_IN_PROGRESS_MARKER: &str = "speech-in-progress";

struct FinishedSpeech {
    settling: SpeechSettling,
    open_check: ThrottledCheck,
    limit: f64,
}

impl FinishedSpeech {
    fn start(entries: Vec<HeardEntry>, now: f64) -> Self {
        Self {
            settling: SpeechSettling::new(entries, now),
            open_check: ThrottledCheck::new(OPEN_CHECK_SECONDS, now),
            limit: now + UNFINISHED_SPEECH_LIMIT_SECONDS,
        }
    }

    fn continues(
        &mut self,
        boundary: &mut (impl Units + Files + Clock),
        watch: &mut ListeningWatch,
    ) -> bool {
        let now = boundary.monotonic();
        now < self.limit
            && self
                .open_check
                .passes(boundary.monotonic(), || watch.still_open(boundary))
    }

    fn settled(&mut self, boundary: &mut (impl Files + Clock)) -> Result<bool, TalkFailure> {
        let heard = last_heard(boundary)?;
        self.settling
            .refresh(read_since(boundary, heard)?, boundary.monotonic());
        let in_progress = boundary.exists(SPEECH_IN_PROGRESS_MARKER);
        Ok(self.settling.settled(boundary.monotonic(), in_progress))
    }
}

pub(super) fn wait_for_finished_speech(
    boundary: &mut (impl Units + Files + Clock),
    entries: Vec<HeardEntry>,
    watch: &mut ListeningWatch,
) -> Result<Vec<HeardEntry>, TalkFailure> {
    let mut speech = FinishedSpeech::start(entries, boundary.monotonic());
    while speech.continues(boundary, watch) && !speech.settled(boundary)? {
        boundary.sleep(POLL_SECONDS);
    }
    Ok(speech.settling.entries().to_vec())
}
