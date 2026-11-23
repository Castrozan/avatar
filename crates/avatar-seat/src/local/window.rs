use super::WINDOW_PORT;
use super::seat::LocalSeat;
use crate::edges::EdgeFailure;
use crate::support::words::joined_words;

const WINDOW_TAIL: [&str; 3] = [
    "--window-size=640,360",
    "--no-first-run",
    "--no-default-browser-check",
];

pub fn hide_window(seat: &LocalSeat<'_>) -> Result<(), EdgeFailure> {
    seat.end_processes_matching(&seat.config.window_profile_flag())
}

pub fn show_window(seat: &LocalSeat<'_>) -> Result<(), EdgeFailure> {
    hide_window(seat)?;
    open_window(seat, "?viewer", WINDOW_PORT)
}

fn window_command(application: &str, flags: &[&str]) -> Vec<String> {
    joined_words(&[
        &["/usr/bin/open", "-n", "-a", application, "--args"],
        flags,
        &WINDOW_TAIL,
    ])
}

fn open_window(seat: &LocalSeat<'_>, page: &str, port: u16) -> Result<(), EdgeFailure> {
    let url = format!("--app=http://localhost:{port}/{page}");
    let profile = seat.config.window_profile_flag();
    let flags = [profile.as_str(), url.as_str()];
    seat.run_checked(&window_command(&seat.config.browser_application, &flags))
}
