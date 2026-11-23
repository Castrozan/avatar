use std::os::unix::process::ExitStatusExt;
use std::process::{Child, Command, ExitStatus};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

use avatar_seat::edges::EdgeFailure;
use avatar_seat::holder::TunnelProcess;
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use super::log::SeatLog;

const POLL: Duration = Duration::from_millis(100);

pub(super) struct ChildTunnel<'run> {
    child: Child,
    terminated: &'run AtomicBool,
}

fn os_failure(error: &impl ToString) -> EdgeFailure {
    EdgeFailure::Os(error.to_string())
}

fn code_of(status: ExitStatus) -> Option<i32> {
    status
        .code()
        .or_else(|| status.signal().map(|number| -number))
}

impl<'run> ChildTunnel<'run> {
    pub(super) fn spawn(
        command: &[String],
        log: &SeatLog,
        terminated: &'run AtomicBool,
    ) -> Result<Self, EdgeFailure> {
        let (program, arguments) = command
            .split_first()
            .ok_or_else(|| EdgeFailure::Os("empty command".to_owned()))?;
        let mut prepared = Command::new(program);
        prepared.args(arguments).stderr(log.tunnel_stderr());
        let child = prepared.spawn().map_err(|error| os_failure(&error))?;
        Ok(Self { child, terminated })
    }

    fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    fn is_open(&mut self) -> bool {
        self.is_running() && !self.terminated.load(Ordering::SeqCst)
    }
}

fn wait_while_open(tunnel: &mut ChildTunnel<'_>, deadline: Instant) {
    while tunnel.is_open() && Instant::now() < deadline {
        sleep(POLL);
    }
}

impl TunnelProcess for ChildTunnel<'_> {
    fn stays_open_for(&mut self, seconds: u64) -> bool {
        wait_while_open(self, Instant::now() + Duration::from_secs(seconds));
        self.is_open()
    }

    fn exit_code(&mut self) -> Option<i32> {
        self.child.try_wait().ok().flatten().and_then(code_of)
    }

    fn terminate(&mut self) {
        let pid = i32::try_from(self.child.id()).ok().map(Pid::from_raw);
        let _ = pid.map(|process| kill(process, Signal::SIGTERM));
    }
}
