pub trait CharacterSelection: Send + Sync {
    fn character_names(&self) -> Vec<String>;

    fn select_character(&self, name: &str) -> bool;
}
