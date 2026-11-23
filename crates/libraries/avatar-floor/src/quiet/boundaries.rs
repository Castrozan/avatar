use super::purpose::ListenerPurpose;
use super::seat::Seat;

pub trait QuietBoundaries {
    fn quiet_seconds(&self) -> f64;
    fn listening_purpose(&self) -> ListenerPurpose;
    fn open_seat(&self) -> Seat;
    fn discord_call_ended(&self) -> bool;
    fn report(&self, message: &str);
    fn close_conversation(&self);
}
