#[derive(Debug, Clone, PartialEq)]
pub struct ThrottledCheck {
    interval_seconds: f64,
    next_check_at: f64,
    last_answer: bool,
}

impl ThrottledCheck {
    #[must_use]
    pub fn starting_at(now_seconds: f64, interval_seconds: f64) -> Self {
        Self {
            interval_seconds,
            next_check_at: now_seconds,
            last_answer: true,
        }
    }
}

pub fn passes(
    throttle: &mut ThrottledCheck,
    now_seconds: f64,
    check: impl FnOnce() -> bool,
) -> bool {
    if now_seconds >= throttle.next_check_at {
        throttle.last_answer = check();
        throttle.next_check_at = now_seconds + throttle.interval_seconds;
    }
    throttle.last_answer
}
