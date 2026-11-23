use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

pub trait Stopping {
    fn is_set(&self) -> bool;
    fn wait(&self, seconds: f64);
}

#[derive(Clone, Default)]
pub struct StopFlag {
    state: Arc<(Mutex<bool>, Condvar)>,
}

impl StopFlag {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&self) {
        let (flag, wake) = &*self.state;
        let _ = flag.lock().map(|mut stopped| *stopped = true);
        wake.notify_all();
    }
}

impl Stopping for StopFlag {
    fn is_set(&self) -> bool {
        self.state.0.lock().is_ok_and(|stopped| *stopped)
    }

    fn wait(&self, seconds: f64) {
        sleep_until_set(&self.state, Duration::from_secs_f64(seconds));
    }
}

fn sleep_until_set(state: &(Mutex<bool>, Condvar), patience: Duration) {
    let Ok(stopped) = state.0.lock() else {
        return;
    };
    let _ = state.1.wait_timeout_while(stopped, patience, |set| !*set);
}
