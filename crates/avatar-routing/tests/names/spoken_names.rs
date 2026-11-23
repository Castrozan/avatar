use avatar_routing::names::spoken_names::{answered_names, called_name};

#[test]
fn nickname_table_extends_called_name() {
    assert_eq!(answered_names("Katsushika"), vec!["Katsushika", "Hokusai"]);
    assert_eq!(
        answered_names("Ozymandias"),
        vec!["Ozymandias", "Ozzy", "Ozzie"]
    );
    assert_eq!(
        answered_names("Scheherazade"),
        vec!["Scheherazade", "Shera", "Shira"]
    );
}

#[test]
fn every_table_entry_matches_python() {
    assert_eq!(answered_names("Xuanzang"), vec!["Xuanzang", "Sanzang"]);
    assert_eq!(
        answered_names("Ushiwakamaru"),
        vec!["Ushiwakamaru", "Ushiwaka"]
    );
    assert_eq!(
        answered_names("Quetzalcoatl"),
        vec!["Quetzalcoatl", "Quetzal"]
    );
    assert_eq!(answered_names("Kingprotea"), vec!["Kingprotea", "Protea"]);
}

#[test]
fn only_the_first_word_is_called() {
    assert_eq!(
        called_name("Scheherazade the Storyteller").as_deref(),
        Some("Scheherazade")
    );
    assert_eq!(answered_names("Ada Lovelace"), vec!["Ada"]);
}

#[test]
fn underscore_splits_words() {
    assert_eq!(called_name("Ozymandias_x").as_deref(), Some("Ozymandias"));
}

#[test]
fn lookup_is_case_sensitive() {
    assert_eq!(answered_names("katsushika"), vec!["katsushika"]);
}

#[test]
fn name_without_nicknames_answers_only_itself() {
    assert_eq!(answered_names("Hokusai"), vec!["Hokusai"]);
}

#[test]
fn unicode_letters_form_words() {
    assert_eq!(called_name("Ünïcode Name").as_deref(), Some("Ünïcode"));
    assert_eq!(answered_names("Ünïcode Name"), vec!["Ünïcode"]);
}

#[test]
fn servant_without_words_answers_nothing() {
    assert_eq!(called_name("  !!  "), None);
    assert!(answered_names("  !!  ").is_empty());
}
