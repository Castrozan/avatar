mod captured;
mod loopback;
mod terminate;
mod timed;

use std::time::Duration;

use avatar_wake::reach::CommandRunner;

pub use captured::{Captured, run_captured};
pub use loopback::loopback_get;
pub use terminate::terminate;
pub use timed::{Completed, run_with_timeout};

pub struct SubprocessRunner;

impl CommandRunner for SubprocessRunner {
    fn stdout_of(&self, command: &[String], timeout_seconds: u64) -> Option<String> {
        let completed = run_with_timeout(command, Duration::from_secs(timeout_seconds))?;
        successful_text(completed)
    }
}

fn successful_text(completed: Completed) -> Option<String> {
    let stdout = completed.succeeded.then_some(completed.stdout)?;
    String::from_utf8(stdout).ok()
}
