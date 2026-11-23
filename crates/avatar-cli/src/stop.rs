use std::io::{self, Write};
use std::process::ExitCode;

use avatar_adapters::commands::{Programs, avatar_stop, run_audio};
use avatar_audio_graph::graph::AudioRequest;

use crate::audio::environment;

fn stop_audio() -> i32 {
    match run_audio(&environment(), || Ok(AudioRequest::Stop)) {
        Ok(state) => i32::from(writeln!(io::stdout().lock(), "{state}").is_err()),
        Err(error) => report(&error.to_string()),
    }
}

fn report(message: &str) -> i32 {
    let _ = writeln!(io::stderr(), "avatar: {message}");
    1
}

pub(crate) fn run() -> ExitCode {
    let status = avatar_stop(&Programs::on_path(), stop_audio);
    ExitCode::from(u8::try_from(status).unwrap_or(1))
}
