use std::time::Duration;

use serde_json::Value;

use crate::session::client::{Outbound, Target};

const LONGEST_VOICE_DELAY_SECONDS: f64 = 2.0;

#[derive(Debug)]
struct PendingLevel {
    due: Duration,
    outbound: Outbound,
}

enum Delivery {
    Now(Outbound),
    Later(PendingLevel),
}

#[derive(Debug, Default)]
struct DelayLane {
    delay: Duration,
    last_due: Duration,
}

impl DelayLane {
    fn schedule(&mut self, now: Duration, to: Target, message: &Value) -> Delivery {
        let due = (now + self.delay).max(self.last_due);
        self.last_due = due;
        let outbound = Outbound {
            to,
            message: message.clone(),
        };
        classify(due, now, outbound)
    }
}

#[derive(Debug, Default)]
pub(crate) struct VoiceLevels {
    viewer: DelayLane,
    stage: DelayLane,
    pending: Vec<PendingLevel>,
}

impl VoiceLevels {
    pub(crate) fn follow_delay(&mut self, viewer: Option<&Value>, stage: Option<&Value>) {
        self.viewer.delay = plausible_delay(viewer).unwrap_or(self.viewer.delay);
        self.stage.delay = plausible_delay(stage).unwrap_or(self.stage.delay);
    }

    pub(crate) fn forget_delay(&mut self) {
        self.viewer.delay = Duration::ZERO;
        self.stage.delay = Duration::ZERO;
    }

    pub(crate) fn send(&mut self, now: Duration, message: &Value) -> Vec<Outbound> {
        let viewer = self.viewer.schedule(now, Target::Viewers, message);
        let stage = self.stage.schedule(now, Target::Stage, message);
        [viewer, stage]
            .into_iter()
            .filter_map(|delivery| queue(&mut self.pending, delivery))
            .collect()
    }

    pub(crate) fn tick(&mut self, now: Duration) -> Vec<Outbound> {
        self.pending.sort_by_key(|level| level.due);
        let due_count = self.pending.partition_point(|level| level.due <= now);
        self.pending
            .drain(..due_count)
            .map(|level| level.outbound)
            .collect()
    }
}

fn queue(pending: &mut Vec<PendingLevel>, delivery: Delivery) -> Option<Outbound> {
    match delivery {
        Delivery::Now(outbound) => Some(outbound),
        Delivery::Later(level) => {
            pending.push(level);
            None
        }
    }
}

fn classify(due: Duration, now: Duration, outbound: Outbound) -> Delivery {
    if due <= now {
        return Delivery::Now(outbound);
    }
    Delivery::Later(PendingLevel { due, outbound })
}

fn plausible_delay(seconds: Option<&Value>) -> Option<Duration> {
    seconds
        .and_then(Value::as_f64)
        .filter(|value| (0.0..=LONGEST_VOICE_DELAY_SECONDS).contains(value))
        .and_then(|value| Duration::try_from_secs_f64(value).ok())
}
