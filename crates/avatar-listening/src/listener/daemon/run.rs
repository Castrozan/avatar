use std::collections::BTreeMap;
use std::sync::Arc;
use std::thread;

use super::configuration::ListenerConfiguration;
use super::kit::ListenerKit;
use super::recording::SegmentRecorder;
use super::segments::{PendingWork, SharedNotes, SharedPending};
use super::segments::{QueueDesk, segment_queue, write_transcript};
use super::timing::wall_seconds_now;
use crate::listener::capture::{SourceSpeech, StopFlag, capture_source};
use crate::listener::diagnostics::Diagnostics;

pub struct DaemonRequest<Kit, Work, Notes> {
    pub kit: Kit,
    pub configuration: ListenerConfiguration,
    pub pending: Arc<Work>,
    pub stopping: StopFlag,
    pub notes: Notes,
}

struct Capturing<'a, Kit, Work, Notes> {
    kit: &'a Kit,
    desk: &'a QueueDesk<Work, Notes>,
    stopping: &'a StopFlag,
}

type Speech<Kit, Work, Notes> = SourceSpeech<
    <Kit as ListenerKit>::Judge,
    <Kit as ListenerKit>::Hearing,
    QueueDesk<Work, Notes>,
>;

impl<'a, Kit, Work, Notes> Capturing<'a, Kit, Work, Notes> {
    fn new(kit: &'a Kit, desk: &'a QueueDesk<Work, Notes>, stopping: &'a StopFlag) -> Self {
        Self {
            kit,
            desk,
            stopping,
        }
    }
}

fn new_speech<Kit: ListenerKit, Work: PendingWork, Notes: Diagnostics + Clone>(
    capturing: &Capturing<Kit, Work, Notes>,
    source: &str,
) -> Speech<Kit, Work, Notes> {
    let desk = capturing.desk.clone();
    let (judge, hearing) = (capturing.kit.judge(), capturing.kit.hearing());
    SourceSpeech::new(source, judge, hearing, desk, wall_seconds_now)
}

fn capture_one<Kit: ListenerKit, Work: PendingWork, Notes: Diagnostics + Clone>(
    capturing: &Capturing<Kit, Work, Notes>,
    source: &str,
    device: &str,
) {
    let mut launcher = capturing.kit.launcher();
    capture_source(device, &mut launcher, capturing.stopping, || {
        new_speech(capturing, source)
    });
}

fn spawn_captures<'scope, 'env: 'scope, Kit, Work, Notes>(
    scope: &'scope thread::Scope<'scope, 'env>,
    capturing: &'scope Capturing<'env, Kit, Work, Notes>,
    sources: &'env BTreeMap<String, String>,
) where
    Kit: ListenerKit,
    Work: PendingWork + Send + Sync,
    Notes: Diagnostics + Clone + Send + Sync,
{
    for (source, device) in sources {
        scope.spawn(move || capture_one(capturing, source, device));
    }
}

pub fn run_daemon<Kit: ListenerKit, Work: SharedPending, Notes: SharedNotes>(
    daemon: &mut DaemonRequest<Kit, Work, Notes>,
    recorder: &mut impl SegmentRecorder,
) {
    let (desk, mut receiver) = segment_queue(Arc::clone(&daemon.pending), daemon.notes.clone());
    let capturing = Capturing::new(&daemon.kit, &desk, &daemon.stopping);
    thread::scope(|scope| {
        spawn_captures(scope, &capturing, &daemon.configuration.sources);
        let (stopping, pending) = (&daemon.stopping, daemon.pending.as_ref());
        let notes = &mut daemon.notes;
        write_transcript(recorder, &mut receiver, stopping, pending, notes);
    });
}
