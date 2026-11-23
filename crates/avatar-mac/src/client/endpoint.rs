use crate::failure::Failure;

pub trait StreamConnection {
    fn send(&mut self, data: &[u8]) -> Result<(), Failure>;
    fn close(&mut self);
}

pub trait SeatEndpoint {
    type Connection: StreamConnection;

    fn attached(&mut self) -> bool;
    fn hush(&mut self);
    fn play(&mut self, pcm: &[u8]) -> Result<(), Failure>;
    fn wait_until_attached(&mut self, seconds: f64, message: &str) -> Result<(), Failure>;
    fn connect_stream(&mut self) -> Result<Self::Connection, Failure>;
}
