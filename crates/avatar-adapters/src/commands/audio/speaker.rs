use std::io;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use avatar_live::boundaries::Speaker;

use super::playback::{RoomSink, chunk_bytes, playback_command};
use super::speaker_queue::SpeakerQueue;
use crate::commands::Programs;
use crate::process::terminate;

const CHUNK_SECONDS: f64 = 0.1;

pub struct PacatSpeaker {
    queue: Arc<SpeakerQueue>,
    players: Vec<Child>,
    feeder: Option<JoinHandle<()>>,
    chunk_bytes: usize,
}

impl PacatSpeaker {
    pub fn start(programs: &Programs, rate: u32, sinks: &[RoomSink]) -> io::Result<Self> {
        let mut players = spawn_players(programs, rate, sinks)?;
        let queue = Arc::new(SpeakerQueue::default());
        let feeder = start_feeder(&queue, take_inputs(&mut players));
        Ok(Self {
            queue,
            players,
            feeder: Some(feeder),
            chunk_bytes: chunk_bytes(rate),
        })
    }

    #[must_use]
    pub fn failure(&self) -> Option<io::ErrorKind> {
        self.queue.failure()
    }

    pub fn close(&mut self) {
        self.queue.close();
        self.players.iter_mut().for_each(terminate);
        let _ = self.feeder.take().map(JoinHandle::join);
    }
}

fn take_inputs(players: &mut [Child]) -> Vec<ChildStdin> {
    players
        .iter_mut()
        .filter_map(|player| player.stdin.take())
        .collect()
}

fn start_feeder(queue: &Arc<SpeakerQueue>, inputs: Vec<ChildStdin>) -> JoinHandle<()> {
    let feeding = Arc::clone(queue);
    thread::spawn(move || feeding.feed(inputs))
}

fn spawn_players(programs: &Programs, rate: u32, sinks: &[RoomSink]) -> io::Result<Vec<Child>> {
    sinks
        .iter()
        .map(|sink| spawn_player(&playback_command(programs, rate, *sink)))
        .collect()
}

fn spawn_player(command: &[String]) -> io::Result<Child> {
    Command::new(&command[0])
        .args(&command[1..])
        .stdin(Stdio::piped())
        .spawn()
}

impl Speaker for PacatSpeaker {
    fn play(&mut self, pcm: &[u8]) {
        let chunks = pcm.chunks(self.chunk_bytes).map(<[u8]>::to_vec);
        self.queue.push(chunks.collect());
    }

    fn buffered_seconds(&self) -> f64 {
        let queued = u32::try_from(self.queue.buffered()).unwrap_or(u32::MAX);
        f64::from(queued) * CHUNK_SECONDS
    }

    fn clear(&mut self) {
        self.queue.clear();
    }
}

impl Drop for PacatSpeaker {
    fn drop(&mut self) {
        self.close();
    }
}
