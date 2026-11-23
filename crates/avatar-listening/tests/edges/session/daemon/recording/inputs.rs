use std::collections::VecDeque;

use crate::support::{count, rows, text};
use avatar_listening::listener::capture::HeardMoments;
use avatar_listening::listener::daemon::{QueuedSegment, TranscribeFailure};
use avatar_listening::listener::speaker_identity::Heard;
use serde_json::Value;

pub(super) type Failure = Box<dyn std::error::Error>;
type Outcomes = VecDeque<Result<String, TranscribeFailure>>;

pub(super) struct Inputs {
    pub(super) items: VecDeque<QueuedSegment>,
    pub(super) outcomes: Outcomes,
    pub(super) identities: VecDeque<Heard>,
    pub(super) runs: i64,
}

fn bytes_of(hex: &str) -> Result<Vec<u8>, Failure> {
    let pair = |index: usize| Ok(u8::from_str_radix(&hex[index..index + 2], 16)?);
    (0..hex.len()).step_by(2).map(pair).collect()
}

fn moments_of(item: &Value) -> Result<HeardMoments, Failure> {
    let speech_ended = item[3].as_f64().ok_or("ended")?;
    let segment_closed = item[4].as_f64().ok_or("closed")?;
    Ok(HeardMoments {
        speech_ended,
        segment_closed,
    })
}

fn segment_of(item: &Value) -> Result<QueuedSegment, Failure> {
    Ok(QueuedSegment {
        source: text(&item[0])?.to_owned(),
        audio: bytes_of(text(&item[1])?)?,
        turn_finished: item[2].as_bool().ok_or("finished")?,
        heard: moments_of(item)?,
    })
}

fn outcome_of(pair: &Value) -> Result<Result<String, TranscribeFailure>, Failure> {
    let value = text(&pair[1])?.to_owned();
    Ok(match text(&pair[0])? {
        "ok" => Ok(value),
        "oserror" => Err(TranscribeFailure::Unreachable(value)),
        _ => Err(TranscribeFailure::Failed(value)),
    })
}

fn heard_of(pair: &Value) -> Heard {
    let numbers = pair[1].as_array().into_iter().flatten();
    let embedding = numbers.filter_map(Value::as_f64).collect();
    Heard {
        speaker: pair[0].as_str().map(str::to_owned),
        embedding,
    }
}

fn run_count(items: usize, scenario: &Value) -> Result<i64, Failure> {
    let extra = count(&scenario["extra_iterations"])?;
    Ok(i64::try_from(items)? + i64::try_from(extra)?)
}

impl Inputs {
    pub(super) fn of(scenario: &Value) -> Result<Self, Failure> {
        let items = rows(&scenario["items"])?.iter().map(segment_of);
        let outcomes = rows(&scenario["outcomes"])?.iter().map(outcome_of);
        let items: VecDeque<_> = items.collect::<Result<_, _>>()?;
        let identities = rows(&scenario["identities"])?;
        Ok(Self {
            runs: run_count(items.len(), scenario)?,
            items,
            outcomes: outcomes.collect::<Result<_, _>>()?,
            identities: identities.iter().map(heard_of).collect(),
        })
    }
}
