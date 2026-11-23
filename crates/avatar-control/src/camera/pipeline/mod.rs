mod arguments;
mod diagnostics;
mod writer;

use std::io;
use std::process::{ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::process::{Child, ChildStderr, ChildStdin, Command};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

pub use arguments::VideoDeviceSettings;

type StartedPipeline = (VideoDeviceFramePipeline, UnboundedReceiver<String>);

pub struct VideoDeviceFramePipeline {
    child: Child,
    frames: Option<UnboundedSender<Vec<u8>>>,
    draining: Arc<AtomicBool>,
}

impl VideoDeviceFramePipeline {
    pub fn start(program: &str, settings: &VideoDeviceSettings) -> io::Result<StartedPipeline> {
        attach(spawn_encoder(program, settings)?)
    }

    fn new(child: Child, frames: UnboundedSender<Vec<u8>>, draining: Arc<AtomicBool>) -> Self {
        Self {
            child,
            frames: Some(frames),
            draining,
        }
    }

    #[must_use]
    pub fn write_frame(&self, frame: Vec<u8>) -> bool {
        self.frames
            .as_ref()
            .is_some_and(|frames| claim_draining(&self.draining) && frames.send(frame).is_ok())
    }

    pub fn end(&mut self) {
        self.frames = None;
    }

    pub async fn finished(&mut self) -> io::Result<ExitStatus> {
        self.child.wait().await
    }
}

fn take_encoder_pipes(child: &mut Child) -> io::Result<(ChildStdin, ChildStderr)> {
    match (child.stdin.take(), child.stderr.take()) {
        (Some(stdin), Some(stderr)) => Ok((stdin, stderr)),
        _ => Err(io::Error::other("encoder pipes were not created")),
    }
}

fn attach(mut child: Child) -> io::Result<StartedPipeline> {
    let (stdin, stderr) = take_encoder_pipes(&mut child)?;
    let (frames, frame_receiver) = unbounded_channel();
    let draining = Arc::new(AtomicBool::new(false));
    tokio::spawn(writer::write_frames(
        stdin,
        frame_receiver,
        Arc::clone(&draining),
    ));
    let pipeline = VideoDeviceFramePipeline::new(child, frames, draining);
    Ok((pipeline, diagnostics::forward_diagnostics(stderr)))
}

fn spawn_encoder(program: &str, settings: &VideoDeviceSettings) -> io::Result<Child> {
    Command::new(program)
        .args(settings.encoder_arguments())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
}

fn claim_draining(draining: &AtomicBool) -> bool {
    !draining.swap(true, Ordering::AcqRel)
}
