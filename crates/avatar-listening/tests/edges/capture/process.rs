use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use crate::edges::fake_program::fake_program;
use crate::support::Outcome;
use avatar_listening::listener::capture::{
    CaptureLauncher, CaptureStream, FRAME_BYTES, FrameSink, ProcessCapture, StopFlag, Stopping,
    capture_source,
};

#[derive(Clone, Default)]
struct Counts {
    bytes: Rc<Cell<usize>>,
    abandoned: Rc<Cell<usize>>,
}

struct CountingSink {
    counts: Counts,
    stop: StopFlag,
}

impl FrameSink for CountingSink {
    fn feed(&mut self, frame: &[u8]) {
        self.counts.bytes.set(self.counts.bytes.get() + frame.len());
    }

    fn abandon(&mut self) {
        self.counts.abandoned.set(self.counts.abandoned.get() + 1);
        self.stop.set();
    }
}

type CommandOf = Box<dyn Fn(&str) -> Vec<String>>;
type Launcher = ProcessCapture<CommandOf>;
type Printing = (tempfile::TempDir, Launcher);

fn launcher_printing(bytes: usize) -> Result<Printing, std::io::Error> {
    let directory = tempfile::tempdir()?;
    let body = format!("head -c {bytes} /dev/zero");
    let program = fake_program(directory.path(), "capture", &body)?;
    let command_of: CommandOf = Box::new(move |_| vec![program.display().to_string()]);
    Ok((directory, ProcessCapture::new(command_of)))
}

fn sink_of(counts: &Counts, stop: &StopFlag) -> CountingSink {
    CountingSink {
        counts: counts.clone(),
        stop: stop.clone(),
    }
}

#[test]
fn a_capture_process_is_read_in_whole_frames_then_a_short_tail() -> Outcome {
    let (_directory, mut launcher) = launcher_printing(2500)?;
    let mut stream = launcher.launch("headset")?;
    let frames = std::iter::from_fn(|| stream.read_frame());
    let lengths: Vec<usize> = frames.map(|frame| frame.len()).collect();
    stream.end();
    assert_eq!(lengths, [FRAME_BYTES, FRAME_BYTES, 2500 - 2 * FRAME_BYTES]);
    Ok(())
}

#[test]
fn the_capture_loop_feeds_abandons_and_stops_when_told() -> Outcome {
    let (_directory, mut launcher) = launcher_printing(2048)?;
    let (stop, counts) = (StopFlag::new(), Counts::default());
    capture_source("headset", &mut launcher, &stop, || sink_of(&counts, &stop));
    assert_eq!((counts.bytes.get(), counts.abandoned.get()), (2048, 1));
    assert!(stop.is_set());
    Ok(())
}

fn stop_after_a_moment(stop: &StopFlag) {
    let waiter = stop.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        waiter.set();
    });
}

#[test]
fn a_capture_that_cannot_start_is_retried_until_stopped() {
    let missing: CommandOf = Box::new(|_| vec!["/nonexistent/parec".to_owned()]);
    let (stop, counts) = (StopFlag::new(), Counts::default());
    stop_after_a_moment(&stop);
    capture_source("headset", &mut ProcessCapture::new(missing), &stop, || {
        sink_of(&counts, &stop)
    });
    assert_eq!(counts.abandoned.get(), 0);
    assert!(stop.is_set());
}
