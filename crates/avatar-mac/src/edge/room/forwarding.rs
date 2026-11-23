use super::{MacRoom, StandardConsole};
use crate::failure::Failure;
use crate::session::Console;
use crate::stage::{StageSurface, StageUnits};
use crate::waiting::Clock;

impl StageUnits for MacRoom {
    fn unit_active(&mut self, unit: &str) -> bool {
        self.stage.units.unit_active(unit)
    }

    fn start_unit(&mut self, unit: &str) -> Result<(), Failure> {
        self.stage.units.start_unit(unit)
    }

    fn stop_unit(&mut self, unit: &str) -> Result<(), Failure> {
        self.stage.units.stop_unit(unit)
    }
}

impl StageSurface for MacRoom {
    fn answers(&mut self, port: u16) -> bool {
        self.stage.surface.answers(port)
    }

    fn hide_window_surface(&mut self) -> Result<(), Failure> {
        self.stage.surface.hide_window_surface()
    }

    fn open_window(&mut self, page: &str, port: u16) -> Result<(), Failure> {
        self.stage.surface.open_window(page, port)
    }

    fn avatar_command(
        &mut self,
        arguments: &[String],
        timeout_seconds: f64,
    ) -> Result<(), Failure> {
        self.stage
            .surface
            .avatar_command(arguments, timeout_seconds)
    }
}

impl Clock for MacRoom {
    fn monotonic(&self) -> f64 {
        self.stage.waiting.monotonic()
    }

    fn sleep(&mut self, seconds: f64) {
        self.stage.waiting.sleep(seconds);
    }
}

impl Console for MacRoom {
    fn print_line(&mut self, line: &str) {
        StandardConsole.print_line(line);
    }

    fn print_error(&mut self, line: &str) {
        StandardConsole.print_error(line);
    }
}
