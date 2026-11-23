use avatar_listening::listener::capture::{StopFlag, Stopping};
use avatar_listening::listener::daemon::stop_on_sigterm;
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

#[test]
fn a_sigterm_sets_the_stop_flag_without_ending_the_process()
-> Result<(), Box<dyn std::error::Error>> {
    let stopping = StopFlag::new();
    stop_on_sigterm(stopping.clone())?;
    assert!(!stopping.is_set());
    kill(Pid::this(), Signal::SIGTERM)?;
    stopping.wait(10.0);
    assert!(stopping.is_set());
    Ok(())
}
