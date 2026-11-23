use super::COMMAND_TIMEOUT_SECONDS;
use super::SEAT_PORT;
use super::failure::SeatFailure;
use super::seat::LocalSeat;
use crate::edges::EdgeFailure;
use crate::support::words::joined_words;

const OPEN_PREFIX: [&str; 4] = ["/usr/bin/open", "-g", "-n", "-a"];
const HEADLESS: [&str; 2] = ["--args", "--headless=new"];
const OPEN_SUFFIX: [&str; 4] = [
    "--use-fake-ui-for-media-stream",
    "--autoplay-policy=no-user-gesture-required",
    "--no-first-run",
    "--no-default-browser-check",
];

fn headless_command(application: &str, profile_flag: &str, page: &str) -> Vec<String> {
    let page_flags = [profile_flag, page];
    joined_words(&[
        &OPEN_PREFIX,
        &[application],
        &HEADLESS,
        &page_flags,
        &OPEN_SUFFIX,
    ])
}

impl LocalSeat<'_> {
    pub(crate) fn open_browser(&self, query: &str) -> Result<(), SeatFailure> {
        let page = format!("--app=http://localhost:{SEAT_PORT}/{query}");
        let profile = self.config.seat_profile_flag();
        let command = headless_command(&self.config.browser_application, &profile, &page);
        Ok(self.run_checked(&command)?)
    }

    pub(crate) fn run_checked(&self, command: &[String]) -> Result<(), EdgeFailure> {
        let outcome = self
            .processes
            .run(command, COMMAND_TIMEOUT_SECONDS, false)?;
        match outcome.code {
            0 => Ok(()),
            code => Err(EdgeFailure::NonZeroExit {
                command: command.to_vec(),
                code,
            }),
        }
    }
}
