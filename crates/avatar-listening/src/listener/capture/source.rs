use std::io;

use super::stop_flag::Stopping;

pub const CAPTURE_RETRY_SECONDS: f64 = 1.0;

pub trait CaptureStream {
    fn read_frame(&mut self) -> Option<Vec<u8>>;
    fn end(&mut self);
}

pub trait CaptureLauncher {
    type Stream: CaptureStream;
    fn launch(&mut self, device: &str) -> io::Result<Self::Stream>;
}

pub trait FrameSink {
    fn feed(&mut self, frame: &[u8]);
    fn abandon(&mut self);
}

fn pump(stream: &mut impl CaptureStream, sink: &mut impl FrameSink, stopping: &impl Stopping) {
    while !stopping.is_set() {
        match stream.read_frame() {
            Some(frame) => sink.feed(&frame),
            None => return,
        }
    }
}

fn capture_once<Sink: FrameSink>(
    stream: &mut impl CaptureStream,
    sink: &mut Sink,
    stopping: &impl Stopping,
) {
    pump(stream, sink, stopping);
    sink.abandon();
    stream.end();
}

pub fn capture_source<Launcher: CaptureLauncher, Sink: FrameSink>(
    device: &str,
    launcher: &mut Launcher,
    stopping: &impl Stopping,
    mut new_sink: impl FnMut() -> Sink,
) {
    while !stopping.is_set() {
        let _ = launcher
            .launch(device)
            .map(|mut stream| capture_once(&mut stream, &mut new_sink(), stopping));
        stopping.wait(CAPTURE_RETRY_SECONDS);
    }
}
