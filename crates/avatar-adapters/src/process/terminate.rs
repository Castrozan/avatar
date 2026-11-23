use std::process::Child;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

pub fn terminate(child: &mut Child) {
    let _ = i32::try_from(child.id()).map(|pid| kill(Pid::from_raw(pid), Signal::SIGTERM));
    let _ = child.wait();
}
