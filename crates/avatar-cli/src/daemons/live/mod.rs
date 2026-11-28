mod agent;
mod devices;
mod records;
mod room;
mod session;
mod shutdown;
mod voice;

use std::io;
use std::process::ExitCode;

use avatar_assets::ProcessEnvironment;
use avatar_live::session::API;
use clap::Args;
use tokio::runtime::Builder;
use tokio::task::LocalSet;

use crate::support::elevenlabs_key::elevenlabs_api_key;
use crate::support::runtime::session_directory;
use session::Environment;

#[derive(Args)]
pub(crate) struct LiveOptions {
    #[arg(long, hide = true, default_value = API)]
    endpoint: String,
}

const MISSING_KEY: &str =
    "no ElevenLabs API key: set ELEVENLABS_API_KEY or write ~/.secrets/elevenlabs-api-key";

fn environment_of(options: &LiveOptions) -> io::Result<Environment> {
    let api_key = elevenlabs_api_key(&ProcessEnvironment);
    if api_key.is_empty() {
        return Err(io::Error::other(MISSING_KEY));
    }
    Ok(Environment {
        directory: session_directory(),
        api_key,
        endpoint: options.endpoint.clone(),
    })
}

pub(crate) fn run(options: &LiveOptions) -> io::Result<ExitCode> {
    let environment = environment_of(options)?;
    let runtime = Builder::new_current_thread().enable_all().build()?;
    let local = LocalSet::new();
    runtime.block_on(local.run_until(session::run(environment)))?;
    Ok(ExitCode::SUCCESS)
}
