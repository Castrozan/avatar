use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

use avatar_seat::local::SeatConfig;

use super::command::run_checked;
use super::processes::BrowserProcesses;
use crate::failure::Failure;
use crate::stage::StageSurface;

const PROBE_TIMEOUT: Duration = Duration::from_secs(1);
const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const WINDOW_TAIL: [&str; 3] = [
    "--window-size=640,360",
    "--no-first-run",
    "--no-default-browser-check",
];

#[derive(Debug, Clone)]
pub struct MacSurface {
    pub avatar_command: PathBuf,
    pub open: PathBuf,
    pub config: SeatConfig,
    pub processes: BrowserProcesses,
}

impl MacSurface {
    fn window_arguments(&self, page: &str, port: u16) -> Vec<String> {
        let profile = self.config.window_profile_flag();
        let url = format!("--app=http://localhost:{port}/{page}");
        let application = &self.config.browser_application;
        let head = ["-n", "-a", application, "--args", &profile, &url];
        head.into_iter()
            .chain(WINDOW_TAIL)
            .map(str::to_owned)
            .collect()
    }
}

impl StageSurface for MacSurface {
    fn answers(&mut self, port: u16) -> bool {
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        TcpStream::connect_timeout(&address, PROBE_TIMEOUT).is_ok()
    }

    fn hide_window_surface(&mut self) -> Result<(), Failure> {
        self.processes
            .end_matching(&self.config.window_profile_flag())
    }

    fn open_window(&mut self, page: &str, port: u16) -> Result<(), Failure> {
        let arguments = self.window_arguments(page, port);
        run_checked(&self.open, &arguments, COMMAND_TIMEOUT)
    }

    fn avatar_command(
        &mut self,
        arguments: &[String],
        timeout_seconds: f64,
    ) -> Result<(), Failure> {
        let timeout = Duration::from_secs_f64(timeout_seconds);
        run_checked(&self.avatar_command, arguments, timeout)
    }
}
