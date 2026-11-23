use avatar_routing::addressing::decision::{Verdict, verdict};
use serde_json::{Value, json};

use super::boundaries::AddressingBoundaries;
use super::decisions::DecisionUnavailable;
use super::utterance::Utterance;

pub(super) struct DecisionReport<'a> {
    boundaries: &'a dyn AddressingBoundaries,
    utterance: &'a Utterance,
    names: &'a [String],
    started: f64,
}

impl<'a> DecisionReport<'a> {
    pub(super) fn begin(
        boundaries: &'a dyn AddressingBoundaries,
        utterance: &'a Utterance,
        names: &'a [String],
    ) -> Self {
        let started = boundaries.now();
        Self {
            boundaries,
            utterance,
            names,
            started,
        }
    }

    pub(super) fn answered(&self, answers: &Value) -> Verdict {
        let decided = verdict(answers, self.names);
        let outcome = json!({"verdict": decided.route, "choice": decided.choice, "confidence": decided.confidence});
        self.record(&outcome);
        decided
    }

    pub(super) fn unavailable(&self, unavailable: &DecisionUnavailable) {
        self.record(&json!({"fallback": unavailable.reason}));
    }

    fn record(&self, outcome: &Value) {
        let elapsed = self.boundaries.now() - self.started;
        let record = decision_record(self.utterance, self.names, elapsed, outcome);
        self.boundaries.report_record(&record);
    }
}

fn decision_record(
    utterance: &Utterance,
    names: &[String],
    elapsed: f64,
    outcome: &Value,
) -> Value {
    let mut record = decision_fields(utterance, names, elapsed);
    if let Value::Object(fields) = &mut record {
        fields.extend(outcome.as_object().cloned().unwrap_or_default());
    }
    record
}

fn decision_fields(utterance: &Utterance, names: &[String], elapsed: f64) -> Value {
    json!({
        "phase": utterance.phase.name(),
        "decision": "addressing",
        "sequence": utterance.sequence,
        "candidates": names,
        "latency_ms": whole_milliseconds(elapsed),
    })
}

pub(crate) fn whole_milliseconds(seconds: f64) -> Value {
    let rounded = format!("{:.0}", seconds * 1000.0);
    json!(rounded.parse::<i64>().unwrap_or_default())
}
