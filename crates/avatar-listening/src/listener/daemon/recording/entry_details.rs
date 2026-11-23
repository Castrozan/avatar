use avatar_transcript::echo::EchoFilter;
use avatar_transcript::json::{Json, JsonObject};
use avatar_transcript::transcript::Utterance;

use crate::listener::daemon::segments::QueuedSegment;
use crate::listener::daemon::timing::Timing;

pub(super) fn details_of(segment: &QueuedSegment, transcription_started: f64) -> JsonObject {
    let mut details = JsonObject::default();
    details.set("turn_finished", Json::Bool(segment.turn_finished));
    details.set("speech_ended", Json::Float(segment.heard.speech_ended));
    details.set("segment_closed", Json::Float(segment.heard.segment_closed));
    details.set("transcription_started", Json::Float(transcription_started));
    details
}

pub(super) fn non_empty(text: String) -> Option<String> {
    (!text.is_empty()).then_some(text)
}

pub(super) fn without_echo(
    echoes: &mut EchoFilter,
    timing: &mut impl Timing,
    source: &str,
    text: String,
) -> Option<String> {
    let repeated = echoes.is_echo(source, &text, timing.wall_seconds());
    (!repeated).then_some(text)
}

pub(super) fn utterance_of(
    segment: &QueuedSegment,
    text: String,
    speaker: Option<String>,
) -> Utterance {
    let source = segment.source.clone();
    Utterance {
        source,
        text,
        speaker,
    }
}

pub(super) fn sequence_of(entry: &Json) -> i64 {
    let Json::Object(object) = entry else {
        return 0;
    };
    match object.get("sequence") {
        Some(Json::Integer(sequence)) => *sequence,
        _ => 0,
    }
}
