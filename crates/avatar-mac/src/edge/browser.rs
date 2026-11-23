use std::path::PathBuf;
use std::time::Duration;

use crate::edge::stage::run_checked;
use crate::failure::Failure;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const OPEN_PROGRAM: &str = "/usr/bin/open";
const FAKE_MEDIA: &str = "--use-fake-ui-for-media-stream";
const NO_FIRST_RUN: &str = "--no-first-run";
const NO_DEFAULT_BROWSER: &str = "--no-default-browser-check";
const AUTOPLAY_FLAG: &str = "--autoplay-policy=no-user-gesture-required";

#[derive(Debug, Clone)]
pub struct HeadlessBrowser {
    pub open: PathBuf,
    pub application: String,
    pub profile_flag: String,
    pub port: u16,
}

impl HeadlessBrowser {
    #[must_use]
    pub fn on_this_mac(application: &str, profile_flag: &str, port: u16) -> Self {
        Self {
            open: PathBuf::from(OPEN_PROGRAM),
            application: application.to_owned(),
            profile_flag: profile_flag.to_owned(),
            port,
        }
    }

    #[must_use]
    pub fn arguments(&self, query: &str, autoplay: bool) -> Vec<String> {
        let page = format!("--app=http://localhost:{}/{query}", self.port);
        let application = &self.application;
        let lead = ["-g", "-n", "-a", application, "--args", "--headless=new"];
        let middle = [&self.profile_flag, &page, FAKE_MEDIA];
        let autoplay = autoplay.then_some(AUTOPLAY_FLAG);
        let tail = [NO_FIRST_RUN, NO_DEFAULT_BROWSER];
        let words = lead.into_iter().chain(middle).chain(autoplay).chain(tail);
        words.map(str::to_owned).collect()
    }

    pub fn open(&self, query: &str, autoplay: bool) -> Result<(), Failure> {
        run_checked(
            &self.open,
            &self.arguments(query, autoplay),
            COMMAND_TIMEOUT,
        )
    }
}
