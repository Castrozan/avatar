use std::error::Error;
use std::path::Path;
use std::sync::Arc;

use avatar_control::speaking::runtime::{
    AgentChannel, AvatarAudioGate, SpeakingPort, SpeechServices, start_speech_runtime,
};
use avatar_control::synthesis::character::{CharacterVoice, CharacterVoiced};
use serde_json::Value;

use super::edges::executables::fake_avatar_audio;
use super::fakes::{FakeChannel, FakeClock, FakeGenerator, FakePlayers};
use super::script::Script;
use super::shared::Shared;

const INITIAL_CLOCK_MS: u64 = 1000;

pub(super) struct World {
    pub(super) script: Arc<Script>,
    pub(super) shared: Shared,
    pub(super) clock: FakeClock,
}

fn channel_for(world: &World, directory: &Path) -> Result<Arc<dyn AgentChannel>, Box<dyn Error>> {
    match &world.script.gate_status {
        Some(status) => Ok(Arc::new(AvatarAudioGate::new(fake_avatar_audio(
            directory, status,
        )?))),
        None => Ok(Arc::new(FakeChannel {
            shared: world.shared.clone(),
            script: Arc::clone(&world.script),
        })),
    }
}

fn services_for(world: &World, directory: &Path) -> Result<SpeechServices, Box<dyn Error>> {
    let players = FakePlayers {
        shared: world.shared.clone(),
        script: Arc::clone(&world.script),
    };
    Ok(SpeechServices {
        clock: Arc::new(world.clock.clone()),
        channel: channel_for(world, directory)?,
        players: Arc::new(players),
        platform: world.script.platform,
    })
}

fn world_for(scenario: &Value) -> World {
    World {
        script: Arc::new(Script::from_scenario(scenario)),
        shared: Shared::new(),
        clock: FakeClock::starting_at(INITIAL_CLOCK_MS),
    }
}

fn generator_for(world: &World) -> FakeGenerator {
    FakeGenerator {
        shared: world.shared.clone(),
        script: Arc::clone(&world.script),
    }
}

pub(super) fn launch(
    scenario: &Value,
    directory: &Path,
) -> Result<(SpeakingPort, World), Box<dyn Error>> {
    let world = world_for(scenario);
    let services = services_for(&world, directory)?;
    Ok((start_speech_runtime(generator_for(&world), services), world))
}

pub(super) fn launch_with_characters(
    scenario: &Value,
    directory: &Path,
    characters: Vec<CharacterVoice>,
) -> Result<(SpeakingPort, World), Box<dyn Error>> {
    let world = world_for(scenario);
    let voiced = CharacterVoiced::new(generator_for(&world), characters)?;
    let services = services_for(&world, directory)?;
    Ok((start_speech_runtime(voiced, services), world))
}
