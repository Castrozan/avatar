use std::path::Path;

use super::command::ProgramCommand;
use super::{AudioPlayer, Playback};

const PLAYBACK_CLIENT_NAME: &str = "avatar-speech";
const NEVER_FALL_BACK_TO_ANOTHER_SINK: &str = "--property=node.dont-fallback=true";

pub struct PulseSinkPlayer {
    command: ProgramCommand,
}

impl PulseSinkPlayer {
    pub fn new(
        program: impl Into<String>,
        sink_name: impl Into<String>,
        runtime_directory: impl Into<String>,
    ) -> Self {
        Self::with_stream_properties(program, sink_name, runtime_directory, Vec::new())
    }

    pub fn only_sink(
        program: impl Into<String>,
        sink_name: impl Into<String>,
        runtime_directory: impl Into<String>,
    ) -> Self {
        let properties = vec![NEVER_FALL_BACK_TO_ANOTHER_SINK.to_owned()];
        Self::with_stream_properties(program, sink_name, runtime_directory, properties)
    }

    fn with_stream_properties(
        program: impl Into<String>,
        sink_name: impl Into<String>,
        runtime_directory: impl Into<String>,
        stream_properties: Vec<String>,
    ) -> Self {
        let arguments = pulse_arguments(sink_name.into(), stream_properties);
        let environment = vec![("XDG_RUNTIME_DIR".to_owned(), runtime_directory.into())];
        Self {
            command: ProgramCommand::new(program.into(), arguments, environment),
        }
    }
}

fn pulse_arguments(sink_name: String, stream_properties: Vec<String>) -> Vec<String> {
    let mut arguments = vec![
        "--client-name".to_owned(),
        PLAYBACK_CLIENT_NAME.to_owned(),
        "--device".to_owned(),
        sink_name,
    ];
    arguments.extend(stream_properties);
    arguments
}

impl AudioPlayer for PulseSinkPlayer {
    fn play(&self, audio_path: &Path) -> Playback {
        self.command.play(audio_path)
    }
}
