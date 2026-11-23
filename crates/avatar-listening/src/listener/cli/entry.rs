use std::collections::BTreeSet;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::Parser;

use super::arguments::ListenArguments;
use super::commands::run;
use super::failure::ListenFailure;
use super::ports::{ListenPorts, Output, PulseSources};
use crate::listener::bounded_command::run_bounded;
use crate::listener::runtime::{Platform, RuntimeEnvironment, runtime_directory};
use crate::listener::session::SessionDirectory;
use crate::listener::units::{LaunchdUnits, SessionUnits, SystemdUnits};
use crate::user_units::SystemClock;
use crate::user_units::launchd::process::ProcessLaunchctl;

const PACTL_TIMEOUT: Duration = Duration::from_secs(30);

struct Pactl;

struct Terminal;

fn pactl(arguments: &[&str]) -> io::Result<String> {
    let output = run_bounded(Path::new("pactl"), arguments, PACTL_TIMEOUT)?;
    if output.code != 0 {
        return Err(io::Error::other(format!("pactl exited {}", output.code)));
    }
    Ok(output.stdout)
}

impl PulseSources for Pactl {
    fn existing_sources(&mut self) -> io::Result<BTreeSet<String>> {
        let listing = pactl(&["list", "short", "sources"])?;
        let names = listing.lines().filter_map(|line| line.split('\t').nth(1));
        Ok(names.map(str::to_owned).collect())
    }

    fn default_microphone(&mut self) -> io::Result<String> {
        Ok(pactl(&["get-default-source"])?.trim().to_owned())
    }
}

impl Output for Terminal {
    fn line(&mut self, text: &str) {
        let mut stdout = io::stdout().lock();
        let _ = writeln!(stdout, "{text}").and_then(|()| stdout.flush());
    }

    fn following(&mut self) -> bool {
        true
    }
}

fn session_directory(platform: Platform, user_identifier: u32) -> SessionDirectory {
    let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_default();
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").ok();
    SessionDirectory::new(runtime_directory(&RuntimeEnvironment {
        platform,
        home: &home,
        runtime_dir: runtime_dir.as_deref(),
        user_identifier,
    }))
}

fn execute(
    arguments: &ListenArguments,
    directory: &SessionDirectory,
    units: &mut dyn SessionUnits,
) -> i32 {
    let mut ports = ListenPorts {
        units,
        pulse: &mut Pactl,
        output: &mut Terminal,
        clock: &mut SystemClock::new(),
    };
    let outcome = run(&arguments.command, directory, &mut ports);
    outcome.map_or_else(|failure| report_failure(&failure), |()| 0)
}

fn report_failure(failure: &ListenFailure) -> i32 {
    let _ = writeln!(io::stderr(), "{failure}");
    1
}

fn execute_on_mac(arguments: &ListenArguments, user_identifier: u32) -> i32 {
    let directory = session_directory(Platform::Mac, user_identifier);
    let mut units = LaunchdUnits::new(ProcessLaunchctl::system(), user_identifier);
    execute(arguments, &directory, &mut units)
}

fn execute_on_linux(arguments: &ListenArguments, user_identifier: u32) -> i32 {
    let directory = session_directory(Platform::Linux, user_identifier);
    execute(arguments, &directory, &mut SystemdUnits::new("systemctl"))
}

pub fn listen_main(arguments: impl IntoIterator<Item = String>) -> i32 {
    let parsed = ListenArguments::try_parse_from(arguments).unwrap_or_else(|error| error.exit());
    let user_identifier = nix::unistd::getuid().as_raw();
    if cfg!(target_os = "macos") {
        return execute_on_mac(&parsed, user_identifier);
    }
    execute_on_linux(&parsed, user_identifier)
}
