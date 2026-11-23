use std::collections::VecDeque;

use avatar_wake::addressing::AddressingState;
use avatar_wake::listener::{Boundaries, Microphone, WakeListener};
use serde_json::Value;

use super::world::World;

const LEVEL: i16 = 1000;
const SAMPLES_PER_FRAME: usize = 512;
const EVENT_LIMIT: usize = 4000;

pub(super) struct Wire<'a> {
    world: &'a World,
    frames: VecDeque<Vec<u8>>,
}

fn frame_of(kind: &str) -> Vec<u8> {
    let level = if kind == "speech" { LEVEL } else { 0 };
    level.to_le_bytes().repeat(SAMPLES_PER_FRAME)
}

fn frames_of(spec: &Value) -> VecDeque<Vec<u8>> {
    let runs = spec.as_array().into_iter().flatten();
    runs.flat_map(|run| {
        let count = usize::try_from(run[1].as_u64().unwrap_or(0)).unwrap_or(0);
        std::iter::repeat_n(frame_of(run[0].as_str().unwrap_or_default()), count)
    })
    .collect()
}

impl<'a> Wire<'a> {
    pub(super) fn new(world: &'a World) -> Self {
        Self {
            world,
            frames: frames_of(&world.config["frames"]),
        }
    }
}

impl Microphone for Wire<'_> {
    fn start(&mut self) {
        self.world.record("microphone_start", Vec::new());
        self.world.starts.set(self.world.starts.get() + 1);
        let allowed = self.world.config["captures"].as_u64().unwrap_or(0);
        let over = self.world.starts.get() > allowed;
        self.world.stopped.set(self.world.stopped.get() || over);
    }

    fn next_frame(&mut self) -> Option<Vec<u8>> {
        let frame = self.frames.pop_front()?;
        let step = self.world.number("frame_clock_step");
        self.world.clock.set(self.world.clock.get() + step);
        Some(frame)
    }

    fn stop(&mut self) {
        self.world.record("microphone_stop", Vec::new());
    }
}

fn listener_of(world: &World) -> WakeListener {
    let dry_run = world.config["dry_run"].as_bool().unwrap_or(false);
    WakeListener::new(dry_run, AddressingState::default())
}

fn drive(world: &World, wire: &mut Wire) {
    let mut listener = listener_of(world);
    let boundaries = Boundaries {
        effects: world,
        seat: world,
    };
    while !world.stopped.get() && world.log.borrow().len() <= EVENT_LIMIT {
        listener.advance(boundaries, wire);
    }
}

pub(super) fn run_events(defaults: &Value, overrides: &Value) -> Vec<Value> {
    let world = World::new(defaults, overrides);
    drive(&world, &mut Wire::new(&world));
    world.log.take()
}
