use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use avatar_assets::{
    DataDirectory, Download, Outcome, ProcessEnvironment, embedded_downloads, install_all,
    parse_downloads,
};
use clap::Args;

use super::transport::HttpTransport;

#[derive(Args)]
pub(crate) struct SetupOptions {
    #[arg(
        long,
        help = "Read the downloads from this manifest file instead of the built-in one"
    )]
    manifest: Option<PathBuf>,
}

pub(crate) fn run(options: &SetupOptions) -> io::Result<()> {
    let downloads = downloads(options)?;
    let data = DataDirectory::locate(&ProcessEnvironment);
    writeln!(
        io::stdout().lock(),
        "data directory: {}",
        data.path().display()
    )?;
    install_all(&downloads, &data, &HttpTransport, &mut announce)
}

fn downloads(options: &SetupOptions) -> io::Result<Vec<Download>> {
    match &options.manifest {
        Some(path) => parse_downloads(&fs::read_to_string(path)?),
        None => embedded_downloads(),
    }
}

fn announce(download: &Download, outcome: Outcome) {
    let state = match outcome {
        Outcome::Installed => "installed",
        Outcome::AlreadyPresent => "already present",
    };
    let _ = writeln!(io::stdout().lock(), "{}: {state}", download.name);
}
