use avatar_floor::addressee::{Addressee, hand_to};
use avatar_floor::claim::{
    Conversation, Listening, RoomQuery, Site, claim, claim_path, parse_room_answer,
};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

pub(crate) const HOST: &str = "thishost";

pub(crate) struct Listens(pub(crate) bool);

impl Listening for Listens {
    fn listening(&self) -> bool {
        self.0
    }
}

#[derive(Default)]
pub(crate) struct Rooms {
    answers: HashMap<String, String>,
}

impl Rooms {
    pub(crate) fn with(mut self, room: &str, answer: &str) -> Self {
        self.answers.insert(room.to_owned(), answer.to_owned());
        self
    }
}

impl RoomQuery for Rooms {
    fn conversation_in_room(&self, room: &str) -> Result<Option<Conversation>, String> {
        match self.answers.get(room) {
            Some(answer) => parse_room_answer(room, answer),
            None => Err(format!("{room} is unreachable")),
        }
    }
}

pub(crate) fn hand_addressee(directory: &Path, pane: &str, host: Option<&str>) -> io::Result<()> {
    let addressee = Addressee {
        pane: pane.to_owned(),
        servant: "Ruri".to_owned(),
        heard_through: 1,
        host: host.map(str::to_owned),
    };
    hand_to(directory, &addressee)
}

pub(crate) fn seed_claim(directory: &Path, pane: &str, text: &str) -> io::Result<()> {
    fs::create_dir_all(directory.join("wake-claims"))?;
    fs::write(claim_path(directory, pane), text)
}

pub(crate) fn read_claim(directory: &Path, pane: &str) -> io::Result<String> {
    fs::read_to_string(claim_path(directory, pane))
}

pub(crate) fn attempt(directory: &Path, pane: &str, room: &str, now: f64) -> io::Result<bool> {
    attempt_among(directory, &Rooms::default(), pane, room, now)
}

pub(crate) fn attempt_among(
    directory: &Path,
    rooms: &Rooms,
    pane: &str,
    room: &str,
    now: f64,
) -> io::Result<bool> {
    let listening = Listens(true);
    let site = Site::new(directory, HOST, &listening, rooms);
    claim(&site, pane, room, now)
}
