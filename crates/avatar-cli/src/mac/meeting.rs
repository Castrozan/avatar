use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use avatar_mac::edge::room::{MacRoom, MacRoomSettings};
use avatar_mac::failure::Failure;
use avatar_mac::meeting::{ListenCommand, carry_out_verb, listen_command, listen_to_the_call};
use avatar_mac::session::Console;
use avatar_seat::local::SeatConfig;
use avatar_seat::meetings::camera::CameraConfig;
use clap::Args;
use nix::unistd::getuid;

use super::environment::{CAMERA_START_SCRIPT, required, seat_config};
use crate::start::listen::run_arguments;
use crate::support::runtime::session_directory;

const OBS_APPLICATION: &str = "/Applications/OBS.app";
const LISTEN_USAGE: &str = "usage: avatar-listen start [-h] [--language LANGUAGE]";

#[derive(Args)]
#[command(disable_help_flag = true, disable_version_flag = true)]
pub(crate) struct MeetingOptions {
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        help = "start, stop or controls"
    )]
    arguments: Vec<String>,
}

#[derive(Args)]
#[command(disable_help_flag = true, disable_version_flag = true)]
pub(crate) struct ListenCallOptions {
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        help = "start and its options, or any avatar listen command"
    )]
    arguments: Vec<String>,
}

fn camera_of(config: &SeatConfig) -> io::Result<CameraConfig> {
    Ok(CameraConfig {
        home: config.home.clone(),
        application: OBS_APPLICATION.to_owned(),
        start_script: required(CAMERA_START_SCRIPT)?,
    })
}

fn settings_of(config: SeatConfig) -> io::Result<MacRoomSettings> {
    Ok(MacRoomSettings {
        camera: camera_of(&config)?,
        config,
        session_directory: session_directory(),
        user_identifier: getuid().as_raw(),
        avatar_command: PathBuf::from("avatar-command"),
        launchctl: PathBuf::from("/bin/launchctl"),
    })
}

fn room() -> io::Result<MacRoom> {
    let settings = settings_of(seat_config()?)?;
    MacRoom::on_this_mac(settings).map_err(|failure| io::Error::other(failure.message))
}

fn code_of(code: i32) -> ExitCode {
    ExitCode::from(u8::try_from(code).unwrap_or(1))
}

fn failed(room: &mut MacRoom, failure: &Failure) -> ExitCode {
    room.print_error(&failure.message);
    ExitCode::FAILURE
}

fn finish(room: &mut MacRoom, outcome: Result<i32, Failure>) -> ExitCode {
    match outcome {
        Ok(code) => code_of(code),
        Err(failure) => failed(room, &failure),
    }
}

pub(super) fn run(options: &MeetingOptions) -> io::Result<ExitCode> {
    let mut room = room()?;
    let outcome = carry_out_verb(&mut room, &options.arguments);
    Ok(finish(&mut room, outcome))
}

fn start_listening(language: Option<&str>) -> io::Result<ExitCode> {
    let mut room = room()?;
    let outcome = listen_to_the_call(&mut room, language).map(|()| 0);
    Ok(finish(&mut room, outcome))
}

fn leave_with(code: i32) -> ExitCode {
    let _ = match code {
        0 => writeln!(io::stdout(), "{LISTEN_USAGE}"),
        _ => writeln!(io::stderr(), "{LISTEN_USAGE}"),
    };
    code_of(code)
}

pub(super) fn listen(options: &ListenCallOptions) -> io::Result<ExitCode> {
    match listen_command(&options.arguments) {
        ListenCommand::Delegate => Ok(run_arguments(&options.arguments)),
        ListenCommand::Start { language } => start_listening(language.as_deref()),
        ListenCommand::Exit(code) => Ok(leave_with(code)),
    }
}
