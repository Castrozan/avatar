use std::future::Future;
use std::pin::pin;

use futures_util::future::{Either, select};

use crate::agent::brief::STOPPED_REASON;

pub(super) enum Ending {
    Answered,
    Failed(String),
    Stopped(f64),
}

impl Ending {
    pub(super) fn of(stopped: bool, seconds: f64, failure: Option<String>) -> Self {
        match (stopped, failure) {
            (true, _) => Self::Stopped(seconds),
            (false, Some(reason)) => Self::Failed(reason),
            (false, None) => Self::Answered,
        }
    }

    pub(super) fn stopped_after(&self) -> Option<f64> {
        match self {
            Self::Stopped(seconds) => Some(*seconds),
            Self::Answered | Self::Failed(_) => None,
        }
    }

    pub(super) fn failure_reason(&self) -> Option<&str> {
        match self {
            Self::Answered => None,
            Self::Failed(reason) => Some(reason),
            Self::Stopped(_) => Some(STOPPED_REASON),
        }
    }
}

pub(super) async fn stopped_first(
    work: impl Future<Output = ()>,
    stop: impl Future<Output = ()>,
) -> bool {
    matches!(select(pin!(work), pin!(stop)).await, Either::Right(_))
}
