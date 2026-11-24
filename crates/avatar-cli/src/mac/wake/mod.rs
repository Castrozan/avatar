use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

use avatar_adapters::microphone::ParecMicrophone;
use avatar_mac::edge::stage::SystemWaiting;
use avatar_mac::edge::wake::MacWakeEdges;
use avatar_mac::talk::opening_where_the_wake_was_heard;
use avatar_seat::local::SEAT_PORT;
use avatar_wake::mac::MacConversation;

mod capture;

use capture::LaunchdCapture;

use super::environment::required;
use super::room::LAUNCHCTL;
use crate::support::wake_log::room_name;
use crate::tools::report;
use crate::wake::{Machine, WakeOptions, dispatch, listen};
use nix::unistd::getuid;

const AVATAR_HOST: &str = "AVATAR_HOST";

fn open_command() -> Vec<String> {
    let talk = std::iter::once("avatar-talk".to_owned());
    talk.chain(opening_where_the_wake_was_heard()).collect()
}

fn edges() -> io::Result<MacWakeEdges> {
    Ok(MacWakeEdges {
        avatar_host: required(AVATAR_HOST)?,
        room: room_name(),
        seat_port: SEAT_PORT,
        ssh: PathBuf::from("ssh"),
        launchctl: PathBuf::from(LAUNCHCTL),
        user_identifier: getuid().as_raw(),
        open_command: open_command(),
        clock: SystemWaiting::new(),
    })
}

fn microphone() -> io::Result<ParecMicrophone> {
    let relay = ["mac", "microphone"].map(str::to_owned).to_vec();
    Ok(ParecMicrophone::relaying(std::env::current_exe()?, relay))
}

fn listen_here(machine: &Machine, dry_run: bool) -> io::Result<ExitCode> {
    let edges = edges()?;
    let mut relay = microphone()?;
    let seat = MacConversation::new(machine, &edges);
    Ok(listen(machine, &seat, &mut relay, dry_run))
}

fn listen_or_report(machine: &Machine, dry_run: bool) -> ExitCode {
    listen_here(machine, dry_run).unwrap_or_else(|error| report(&error))
}

pub(super) fn run(options: &WakeOptions) -> io::Result<ExitCode> {
    let machine = Machine::capturing_with(Box::new(LaunchdCapture::on_this_mac()))?;
    dispatch(options, &machine, listen_or_report)
}
