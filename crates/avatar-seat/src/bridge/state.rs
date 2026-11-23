use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use axum::extract::ws::Message;
use serde_json::Value;
use tokio::sync::{mpsc, watch};

use crate::bridge::tools::AudioTools;
use crate::host::levels::{LevelSource, current};
use crate::host::state::{HostProbe, HostState};
use crate::page::{PageFiles, SeatPage};

pub trait BridgeLog: Send + Sync {
    fn log(&self, message: &str);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SeatPresence {
    pub attachments: u64,
    pub attached: bool,
}

pub struct BridgeSettings {
    pub allowed_origins: Vec<String>,
    pub first_attach: Duration,
    pub reattach: Duration,
}

pub(crate) struct Runtime {
    pub(crate) page: Mutex<SeatPage>,
    pub(crate) presence: watch::Sender<SeatPresence>,
    pub(crate) stopping: watch::Sender<bool>,
    pub(crate) audio_sender: Mutex<Option<mpsc::UnboundedSender<Message>>>,
    pub(crate) meeting_carried: AtomicBool,
}

impl Runtime {
    fn new() -> Self {
        Self {
            page: Mutex::new(SeatPage::new(true)),
            presence: watch::channel(SeatPresence::default()).0,
            stopping: watch::channel(false).0,
            audio_sender: Mutex::new(None),
            meeting_carried: AtomicBool::new(false),
        }
    }
}

pub struct Bridge<Tools: AudioTools, Source: LevelSource, Probe: HostProbe> {
    pub tools: Tools,
    pub levels: Source,
    pub host: HostState<Probe>,
    pub files: Box<dyn PageFiles + Send + Sync>,
    pub settings: BridgeSettings,
    pub log: Box<dyn BridgeLog>,
    pub(crate) runtime: Runtime,
}

pub(crate) fn locked<Guarded>(mutex: &Mutex<Guarded>) -> MutexGuard<'_, Guarded> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

pub type BridgeParts<Tools, Source, Probe> = (Tools, Source, HostState<Probe>);

impl<Tools: AudioTools, Source: LevelSource, Probe: HostProbe> Bridge<Tools, Source, Probe> {
    pub fn new(
        parts: BridgeParts<Tools, Source, Probe>,
        files: Box<dyn PageFiles + Send + Sync>,
        settings: BridgeSettings,
        log: Box<dyn BridgeLog>,
    ) -> Self {
        let (tools, levels, host) = parts;
        Self {
            tools,
            levels,
            host,
            files,
            settings,
            log,
            runtime: Runtime::new(),
        }
    }
}

pub(crate) async fn send_levels<Tools: AudioTools, Source: LevelSource, Probe: HostProbe>(
    bridge: &Bridge<Tools, Source, Probe>,
) {
    let Some(sender) = locked(&bridge.runtime.audio_sender).clone() else {
        return;
    };
    let levels = current(&bridge.levels).await;
    let text = Value::Object(levels).to_string();
    let _ = sender.send(Message::Text(text.into()));
}
