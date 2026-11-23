use super::session::Session;

pub trait FloorBoundaries {
    fn sessions(&self) -> Vec<Session>;
    fn listening(&self) -> bool;
}

pub trait ServantCharacter {
    fn show_for(&self, servant: &str);
}
