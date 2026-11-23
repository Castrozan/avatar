use std::process::Output;
use std::time::Duration;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tokio::process::Child;
use tokio::time::sleep;

const STACK_DUMP_MILLISECONDS: u64 = 500;

pub(super) async fn wait_with_timeout(child: Child, timeout: Duration) -> std::io::Result<Output> {
    let process_identifier = child.id();
    let watchdog = tokio::spawn(stop_after(process_identifier, timeout));
    let output = child.wait_with_output().await;
    watchdog.abort();
    output
}

async fn stop_after(process_identifier: Option<u32>, timeout: Duration) {
    sleep(timeout).await;
    send_signal(process_identifier, Signal::SIGUSR1);
    sleep(Duration::from_millis(STACK_DUMP_MILLISECONDS)).await;
    send_signal(process_identifier, Signal::SIGTERM);
}

fn send_signal(process_identifier: Option<u32>, signal: Signal) {
    let Some(identifier) = process_identifier.and_then(|value| i32::try_from(value).ok()) else {
        return;
    };
    kill(Pid::from_raw(identifier), signal).ok();
}
