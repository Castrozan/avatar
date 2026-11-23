use std::collections::HashMap;

const SEPARATOR: &str = " | ";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListedCharacters {
    by_servant: HashMap<String, String>,
}

impl ListedCharacters {
    #[must_use]
    pub fn character_for(&self, servant: &str) -> Option<&str> {
        self.by_servant.get(servant).map(String::as_str)
    }
}

#[must_use]
pub fn parse_listed_characters(text: &str) -> ListedCharacters {
    let by_servant = text
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .lines()
        .filter_map(|line| line.split_once(SEPARATOR))
        .map(|(name, character)| (name.to_owned(), character.trim().to_owned()))
        .collect();
    ListedCharacters { by_servant }
}
