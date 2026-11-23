use std::process::ExitStatus;

use tokio::io::AsyncReadExt;
use tokio::process::{Child, ChildStderr};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PlaybackReport {
    pub exit_status: Option<ExitStatus>,
    pub stderr: String,
}

pub struct Playback {
    stop_signal: Option<oneshot::Sender<()>>,
    completion: JoinHandle<PlaybackReport>,
}

impl Playback {
    pub(super) fn supervise(child: Child) -> Self {
        let (stop_signal, stop_requested) = oneshot::channel();
        let completion = tokio::spawn(run_to_completion(child, stop_requested));
        Self {
            stop_signal: Some(stop_signal),
            completion,
        }
    }

    pub(super) fn unstarted() -> Self {
        Self {
            stop_signal: None,
            completion: tokio::spawn(std::future::ready(PlaybackReport::default())),
        }
    }

    pub fn stop(&mut self) {
        let _ = self.stop_signal.take().map(|signal| signal.send(()));
    }

    pub async fn finished(self) -> PlaybackReport {
        self.completion.await.unwrap_or_default()
    }
}

enum Settlement {
    Exited(Option<ExitStatus>),
    StopRequested,
}

async fn run_to_completion(
    mut child: Child,
    mut stop_requested: oneshot::Receiver<()>,
) -> PlaybackReport {
    let stderr_reader = tokio::spawn(read_stderr(child.stderr.take()));
    let exit_status = exit_status_of(&mut child, &mut stop_requested).await;
    PlaybackReport {
        exit_status,
        stderr: stderr_reader.await.unwrap_or_default(),
    }
}

async fn exit_status_of(
    child: &mut Child,
    stop_requested: &mut oneshot::Receiver<()>,
) -> Option<ExitStatus> {
    match settle(child, stop_requested).await {
        Settlement::Exited(status) => status,
        Settlement::StopRequested => terminate(child).await,
    }
}

async fn settle(child: &mut Child, stop_requested: &mut oneshot::Receiver<()>) -> Settlement {
    tokio::select! {
        status = child.wait() => Settlement::Exited(status.ok()),
        () = wait_for_stop_request(stop_requested) => Settlement::StopRequested,
    }
}

async fn wait_for_stop_request(stop_requested: &mut oneshot::Receiver<()>) {
    if stop_requested.await.is_err() {
        std::future::pending::<()>().await;
    }
}

async fn terminate(child: &mut Child) -> Option<ExitStatus> {
    let _ = child.start_kill();
    child.wait().await.ok()
}

async fn read_stderr(stderr: Option<ChildStderr>) -> String {
    let Some(mut stderr) = stderr else {
        return String::new();
    };
    let mut bytes = Vec::new();
    let _ = stderr.read_to_end(&mut bytes).await;
    String::from_utf8_lossy(&bytes).trim().to_owned()
}
