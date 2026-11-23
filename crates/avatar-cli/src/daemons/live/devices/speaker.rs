use std::io::{self, Write};
use std::process::{Child, ChildStdin};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use avatar_live::boundaries::Speaker;
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use super::playback::start_playback;
use super::queue::ChunkQueue;
use super::sinks::Sink;
use crate::daemons::live::shutdown::Shutdown;

const CHUNKS_PER_SECOND: u32 = 10;
const SAMPLE_WIDTH_BYTES: usize = 2;

pub(crate) struct PacatSpeaker {
    queue: Arc<ChunkQueue>,
    chunk_bytes: usize,
    players: Vec<Child>,
    feeder: Option<JoinHandle<()>>,
}

fn write_chunk(outputs: &mut [ChildStdin], chunk: &[u8]) -> io::Result<()> {
    outputs.iter_mut().try_for_each(|output| {
        output.write_all(chunk)?;
        output.flush()
    })
}

fn feed(queue: &ChunkQueue, mut outputs: Vec<ChildStdin>, shutdown: &Shutdown) {
    let mut chunks = std::iter::from_fn(|| queue.next());
    if let Err(failure) = chunks.try_for_each(|chunk| write_chunk(&mut outputs, &chunk)) {
        shutdown.fail(format!("The speaker closed: {failure}"));
    }
}

fn spawn_feeder(
    queue: &Arc<ChunkQueue>,
    players: &mut [Child],
    shutdown: Arc<Shutdown>,
) -> JoinHandle<()> {
    let outputs: Vec<ChildStdin> = players
        .iter_mut()
        .filter_map(|player| player.stdin.take())
        .collect();
    let feeding = Arc::clone(queue);
    thread::spawn(move || feed(&feeding, outputs, &shutdown))
}

fn terminate(player: &mut Child) {
    let signal = |pid| kill(Pid::from_raw(pid), Signal::SIGTERM);
    let _ = i32::try_from(player.id()).map(signal);
    let _ = player.wait();
}

fn chunk_bytes(rate: u32) -> usize {
    let samples = usize::try_from(rate / CHUNKS_PER_SECOND).unwrap_or_default();
    samples * SAMPLE_WIDTH_BYTES
}

fn start_players(rate: u32, sinks: &[Sink]) -> io::Result<Vec<Child>> {
    sinks
        .iter()
        .map(|sink| start_playback(rate, sink))
        .collect()
}

impl PacatSpeaker {
    pub(crate) fn start(rate: u32, sinks: &[Sink], shutdown: Arc<Shutdown>) -> io::Result<Self> {
        let mut players = start_players(rate, sinks)?;
        let queue = Arc::new(ChunkQueue::default());
        let feeder = spawn_feeder(&queue, &mut players, shutdown);
        Ok(Self {
            queue,
            chunk_bytes: chunk_bytes(rate),
            players,
            feeder: Some(feeder),
        })
    }
}

impl Speaker for PacatSpeaker {
    fn play(&mut self, pcm: &[u8]) {
        self.queue
            .push_all(pcm.chunks(self.chunk_bytes).map(<[u8]>::to_vec));
    }

    fn buffered_seconds(&self) -> f64 {
        let queued = u32::try_from(self.queue.len()).unwrap_or(u32::MAX);
        f64::from(queued) / f64::from(CHUNKS_PER_SECOND)
    }

    fn clear(&mut self) {
        self.queue.clear();
    }
}

impl Drop for PacatSpeaker {
    fn drop(&mut self) {
        self.queue.close();
        let _ = self.feeder.take().map(JoinHandle::join);
        self.players.iter_mut().for_each(terminate);
    }
}
