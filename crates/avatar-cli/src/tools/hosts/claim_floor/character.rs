use avatar_assets::Asset;
use avatar_floor::floor::ServantCharacter;
use avatar_wake::character::listing::parse_listed_characters;
use avatar_wake::character::switch::ListedServantCharacter;

use crate::wake::machine::Shell;

fn listed_text() -> String {
    avatar_assets::resolve(Asset::ServantCharacters)
        .ok()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .unwrap_or_default()
}

pub(super) fn show_for(servant: &str) {
    ListedCharacter.show_for(servant);
}

pub(super) struct ListedCharacter;

impl ServantCharacter for ListedCharacter {
    fn show_for(&self, servant: &str) {
        let listed = parse_listed_characters(&listed_text());
        ListedServantCharacter::new(listed, &Shell).show_for(servant);
    }
}
