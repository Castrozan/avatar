use std::process::Child;
use std::thread::sleep;
use std::time::{Duration, Instant};

const POLL: Duration = Duration::from_millis(100);

pub(super) fn eventually(mut condition: impl FnMut() -> bool, limit: Duration) -> bool {
    let started = Instant::now();
    while started.elapsed() < limit && !condition() {
        sleep(POLL);
    }
    condition()
}

pub(super) fn exit_code_within(
    mut child: Child,
    limit: Duration,
) -> Result<Option<i32>, Box<dyn std::error::Error>> {
    let exited = eventually(
        || child.try_wait().is_ok_and(|status| status.is_some()),
        limit,
    );
    if exited {
        return Ok(child.wait()?.code());
    }
    child.kill()?;
    child.wait()?;
    Ok(None)
}
