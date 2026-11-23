use crate::failure::Failure;

pub const POLL_SECONDS: f64 = 0.25;

pub trait Clock {
    fn monotonic(&self) -> f64;
    fn sleep(&mut self, seconds: f64);
}

fn pause_before_retry(
    context: &mut impl Clock,
    deadline: f64,
    timeout_message: &str,
) -> Result<(), Failure> {
    if context.monotonic() > deadline {
        return Err(Failure::timeout(timeout_message));
    }
    context.sleep(POLL_SECONDS);
    Ok(())
}

pub fn wait_until<Context: Clock>(
    context: &mut Context,
    seconds: f64,
    timeout_message: &str,
    mut is_ready: impl FnMut(&mut Context) -> Result<bool, Failure>,
) -> Result<(), Failure> {
    let deadline = context.monotonic() + seconds;
    while !is_ready(context)? {
        pause_before_retry(context, deadline, timeout_message)?;
    }
    Ok(())
}
