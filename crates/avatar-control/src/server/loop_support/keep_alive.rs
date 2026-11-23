use std::time::Duration;

use tokio::time::{Instant, Interval, interval_at};

const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(30);

pub(crate) struct KeepAlive {
    ticker: Interval,
    answered: bool,
}

impl KeepAlive {
    pub(crate) fn new() -> Self {
        let start = Instant::now() + KEEP_ALIVE_INTERVAL;
        Self {
            ticker: interval_at(start, KEEP_ALIVE_INTERVAL),
            answered: true,
        }
    }

    pub(crate) async fn tick(&mut self) {
        self.ticker.tick().await;
    }

    pub(crate) fn record_pong(&mut self) {
        self.answered = true;
    }

    pub(crate) fn begin_probe(&mut self) -> bool {
        std::mem::replace(&mut self.answered, false)
    }
}
