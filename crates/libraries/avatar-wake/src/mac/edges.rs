use crate::listener::Boundary;

pub trait MacEdges {
    fn listening_here(&self) -> bool;
    fn seat_listening(&self) -> bool;
    fn avatar_host(&self) -> String;
    fn room_name(&self) -> Option<String>;
    fn at_the_avatar_host(&self, command: &[String], timeout_seconds: u64) -> Boundary<String>;
    fn open_here(&self) -> Boundary<bool>;
    fn clock(&self) -> f64;
}
