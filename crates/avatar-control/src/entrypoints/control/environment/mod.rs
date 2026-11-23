mod port;

use std::io;
use std::path::{Path, PathBuf};

use port::parse_port;

const DEFAULT_WEBSOCKET_PORT: &str = "8765";
const DEFAULT_HTTP_PORT: &str = "8766";
const DEFAULT_RENDERER_PORT: &str = "3000";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlEnvironment {
    pub websocket_port: u16,
    pub http_port: u16,
    pub renderer_port: String,
    pub audio_monitor: Option<String>,
    pub audio_directory: PathBuf,
}

struct Variables<Lookup>(Lookup);

impl<Lookup: Fn(&str) -> Option<String>> Variables<Lookup> {
    fn optional(&self, name: &str) -> Option<String> {
        (self.0)(name).filter(|text| !text.is_empty())
    }

    fn text(&self, name: &str, fallback: &str) -> String {
        self.optional(name).unwrap_or_else(|| fallback.to_owned())
    }

    fn port(&self, name: &str, fallback: &str) -> io::Result<u16> {
        parse_port(name, &self.text(name, fallback))
    }
}

impl ControlEnvironment {
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>, home: &Path) -> io::Result<Self> {
        let variables = Variables(lookup);
        Ok(Self {
            websocket_port: variables.port("AVATAR_WS_PORT", DEFAULT_WEBSOCKET_PORT)?,
            http_port: variables.port("AVATAR_HTTP_PORT", DEFAULT_HTTP_PORT)?,
            renderer_port: variables.text("AVATAR_RENDERER_PORT", DEFAULT_RENDERER_PORT),
            audio_monitor: variables.optional("AVATAR_AUDIO_MONITOR"),
            audio_directory: home.join(".cache").join("clever-avatar").join("tts"),
        })
    }

    pub fn from_process(home: &Path) -> io::Result<Self> {
        Self::from_lookup(|name| std::env::var(name).ok(), home)
    }
}
