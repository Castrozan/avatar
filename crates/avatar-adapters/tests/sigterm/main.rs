use std::time::{Duration, Instant};

use avatar_adapters::holder::StopSignal;
use nix::sys::signal::{Signal, kill};
use nix::unistd::getpid;

#[test]
fn sigterm_raises_the_stop_signal_instead_of_killing_the_process()
-> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Runtime::new()?;
    let stop = StopSignal::on_sigterm(runtime.handle())?;
    assert!(!stop.is_raised());
    kill(getpid(), Signal::SIGTERM)?;
    let deadline = Instant::now() + Duration::from_secs(5);
    while !stop.is_raised() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(stop.is_raised());
    Ok(())
}
