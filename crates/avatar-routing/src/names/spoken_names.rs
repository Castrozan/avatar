use crate::text::folding::word_matches;

const NICKNAMES: &[(&str, &[&str])] = &[
    ("Katsushika", &["Hokusai"]),
    ("Kingprotea", &["Protea"]),
    ("Ozymandias", &["Ozzy", "Ozzie"]),
    ("Quetzalcoatl", &["Quetzal"]),
    ("Scheherazade", &["Shera", "Shira"]),
    ("Ushiwakamaru", &["Ushiwaka"]),
    ("Xuanzang", &["Sanzang"]),
];

#[must_use]
pub fn called_name(servant: &str) -> Option<String> {
    word_matches(servant)
        .into_iter()
        .next()
        .map(|word_match| word_match.text)
}

#[must_use]
pub fn answered_names(servant: &str) -> Vec<String> {
    called_name(servant).map_or_else(Vec::new, with_nicknames)
}

fn with_nicknames(called: String) -> Vec<String> {
    let nicknames = NICKNAMES
        .iter()
        .find(|(name, _)| *name == called)
        .map_or(&[][..], |(_, nicknames)| *nicknames);
    std::iter::once(called)
        .chain(nicknames.iter().map(|nickname| (*nickname).to_owned()))
        .collect()
}
