use std::error::Error;
use std::net::SocketAddr;

use avatar_control::server::{ShutdownHandle, serve};
use avatar_control::session::control_origins;
use avatar_control::speaking::runtime::SpeakingPort;
use avatar_control::synthesis::character::CharacterVoice;
use serde_json::{Value, json};

use super::socket::{Socket, identified, receive, send};
use crate::runtime::harness::{World, launch_with_characters};

pub(super) struct Stage {
    pub(super) url: String,
    pub(super) shutdown: ShutdownHandle,
    world: World,
    _directory: tempfile::TempDir,
}

impl Stage {
    fn holding(url: String, shutdown: ShutdownHandle, scene: (World, tempfile::TempDir)) -> Self {
        Self {
            url,
            shutdown,
            world: scene.0,
            _directory: scene.1,
        }
    }
}

fn character(name: &str, cloud_voice: &str) -> CharacterVoice {
    CharacterVoice {
        name: name.to_owned(),
        cloud_voice: Some(cloud_voice.to_owned()),
        local_voice: None,
    }
}

fn cast() -> Vec<CharacterVoice> {
    vec![
        character("first", "firstCloud"),
        character("second", "secondCloud"),
    ]
}

async fn served(port: SpeakingPort) -> Result<(String, ShutdownHandle), Box<dyn Error>> {
    let loopback = SocketAddr::from(([127, 0, 0, 1], 0));
    let (address, shutdown) = serve(loopback, port, control_origins("3000")).await?;
    Ok((format!("ws://{address}/"), shutdown))
}

pub(super) async fn start() -> Result<Stage, Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let (port, world) = launch_with_characters(&json!({"steps": []}), directory.path(), cast())?;
    let (url, shutdown) = served(port).await?;
    Ok(Stage::holding(url, shutdown, (world, directory)))
}

pub(super) async fn request_character(
    agent: &mut Socket,
    name: &str,
) -> Result<Value, Box<dyn Error>> {
    send(agent, &json!({"type": "setCharacter", "name": name})).await?;
    receive(agent).await
}

pub(super) async fn change_relayed_from_the_renderer(
    stage: &Stage,
    agent: &mut Socket,
) -> Result<Value, Box<dyn Error>> {
    let mut renderer = identified(&stage.url, "renderer").await?;
    let shown = json!({"type": "characterChanged", "name": "second"});
    send(&mut renderer, &shown).await?;
    receive(agent).await
}

fn generated_voices(stage: &Stage) -> Vec<Value> {
    stage.world.shared.with(|recording| {
        let generated = recording
            .trace
            .iter()
            .filter_map(|entry| entry.get("generate"));
        generated
            .map(|generation| generation["voice"].clone())
            .collect()
    })
}

pub(super) async fn spoken_voices(
    stage: &Stage,
    agent: &mut Socket,
) -> Result<Vec<Value>, Box<dyn Error>> {
    send(
        agent,
        &json!({"type": "speak", "text": "Hello", "id": "line"}),
    )
    .await?;
    assert_eq!(receive(agent).await?["type"], "speakAck");
    Ok(generated_voices(stage))
}
