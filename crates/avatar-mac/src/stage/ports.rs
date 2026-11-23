use crate::failure::Failure;
use crate::waiting::Clock;

pub trait StageUnits {
    fn unit_active(&mut self, unit: &str) -> bool;
    fn start_unit(&mut self, unit: &str) -> Result<(), Failure>;
    fn stop_unit(&mut self, unit: &str) -> Result<(), Failure>;
}

pub trait StageSurface {
    fn answers(&mut self, port: u16) -> bool;
    fn hide_window_surface(&mut self) -> Result<(), Failure>;
    fn open_window(&mut self, page: &str, port: u16) -> Result<(), Failure>;
    fn avatar_command(&mut self, arguments: &[String], timeout_seconds: f64)
    -> Result<(), Failure>;
}

pub trait Stage: StageUnits + StageSurface + Clock {}

impl<Candidate: StageUnits + StageSurface + Clock> Stage for Candidate {}
