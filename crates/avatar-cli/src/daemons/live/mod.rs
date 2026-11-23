mod agent;
mod devices;
mod records;
mod room;
mod session;
mod shutdown;
mod voice;

use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

use avatar_live::session::ENDPOINT;
use clap::Args;
use tokio::runtime::Builder;
use tokio::task::LocalSet;

use crate::support::runtime::session_directory;
use session::Environment;

const KEY_FILE: &str = ".secrets/openai-api-key";

#[derive(Args)]
pub(crate) struct LiveOptions {
    #[arg(long, hide = true, default_value = ENDPOINT)]
    endpoint: String,
}

fn environment_of(options: &LiveOptions) -> io::Result<Environment> {
    let home = std::env::var_os("HOME").ok_or_else(|| io::Error::other("HOME is not set"))?;
    Ok(Environment {
        directory: session_directory(),
        key_file: PathBuf::from(home).join(KEY_FILE),
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
