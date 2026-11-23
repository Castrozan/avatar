use std::io;
use std::process::ExitCode;

use avatar_mac::edge::room::{MacHostTalk, MacLiveOpening};
use avatar_mac::talk::{MacTalkRoom, run as run_talk};
use clap::Args;

mod listener;
mod units;

use listener::LaunchdTalkListener;
use units::LaunchdTalkUnits;

use super::environment::holder_config;
use super::room::{code_of, room};
use crate::support::runtime::session_directory;
use crate::talk::TalkBoundaries;

type MacBoundaries = TalkBoundaries<LaunchdTalkUnits, LaunchdTalkListener>;

#[derive(Args)]
#[command(disable_help_flag = true, disable_version_flag = true)]
pub(crate) struct TalkOptions {
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        help = "open, close, status, hear, ask, follow, say, reply, show or hide, then their options"
    )]
    arguments: Vec<String>,
}

fn boundaries() -> MacBoundaries {
    let directory = session_directory();
    let listener = LaunchdTalkListener::on_this_mac(directory.clone());
    TalkBoundaries::with_parts(directory, LaunchdTalkUnits::on_this_mac(), listener)
}

fn host() -> io::Result<MacHostTalk<MacBoundaries>> {
    Ok(MacHostTalk {
        boundary: boundaries(),
        config: holder_config()?,
    })
}

fn run_with(
    seat: &mut impl MacTalkRoom,
    host: &mut MacHostTalk<MacBoundaries>,
    arguments: &[String],
) -> ExitCode {
    let mut live = MacLiveOpening {
        boundary: boundaries(),
    };
    code_of(run_talk(seat, host, &mut live, arguments))
}

pub(super) fn run(options: &TalkOptions) -> io::Result<ExitCode> {
    let mut host = host()?;
    let mut seat = room()?.with_talk(boundaries());
    Ok(run_with(&mut seat, &mut host, &options.arguments))
}
