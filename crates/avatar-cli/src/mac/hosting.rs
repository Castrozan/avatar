use std::io;
use std::process::ExitCode;
use std::time::Duration;

use avatar_mac::edge::browser::HeadlessBrowser;
use avatar_mac::edge::server::{PageFiles, ServerProgram};
use avatar_seat::local::SeatConfig;
use clap::Args;

use super::environment::{SEAT_PAGE, WAKE_PAGE, page_directory, seat_config};
use super::serving::host;

const SEAT_PORT: u16 = 8772;
const MICROPHONE_PORT: u16 = 8771;
const ATTACH_SECONDS: u64 = 30;
const MEETING_QUERY: &str = "?meeting";

#[derive(Args)]
pub(crate) struct SeatOptions {
    #[arg(long, help = "Open the meeting page instead of the conversation page")]
    meeting: bool,
}

#[derive(Clone, Copy)]
pub(super) enum Kind {
    Seat,
    Microphone,
}

pub(super) struct Hosting {
    pub(super) kind: Kind,
    pub(super) program: ServerProgram,
    pub(super) browser: HeadlessBrowser,
    pub(super) query: &'static str,
}

fn program_of(config: &SeatConfig, files: PageFiles, port: u16, subject: &str) -> ServerProgram {
    let browser = config.browser_name();
    ServerProgram {
        port,
        files,
        attach: Duration::from_secs(ATTACH_SECONDS),
        no_page_message: format!(
            "The {subject} page did not attach; check that {browser} may use the microphone"
        ),
    }
}

fn browser_of(config: &SeatConfig, profile: &str, port: u16) -> HeadlessBrowser {
    let flag = format!(
        "--user-data-dir={}/.local/share/avatar/{profile}",
        config.home
    );
    HeadlessBrowser::on_this_mac(&config.browser_application, &flag, port)
}

fn seat_hosting(options: &SeatOptions) -> io::Result<Hosting> {
    let config = seat_config()?;
    let files = PageFiles::seat(&page_directory(SEAT_PAGE)?);
    Ok(Hosting {
        kind: Kind::Seat,
        program: program_of(&config, files, SEAT_PORT, "conversation"),
        browser: browser_of(&config, "conversation-browser", SEAT_PORT),
        query: if options.meeting { MEETING_QUERY } else { "" },
    })
}

fn microphone_hosting() -> io::Result<Hosting> {
    let config = seat_config()?;
    let files = PageFiles::microphone(&page_directory(WAKE_PAGE)?, &page_directory(SEAT_PAGE)?);
    Ok(Hosting {
        kind: Kind::Microphone,
        program: program_of(&config, files, MICROPHONE_PORT, "wake microphone"),
        browser: browser_of(&config, "wake-browser", MICROPHONE_PORT),
        query: "",
    })
}

pub(super) fn run_seat(options: &SeatOptions) -> io::Result<ExitCode> {
    host(seat_hosting(options)?)
}

pub(super) fn run_microphone() -> io::Result<ExitCode> {
    host(microphone_hosting()?)
}
