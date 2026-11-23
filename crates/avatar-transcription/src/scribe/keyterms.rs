use avatar_routing::names::spoken_names::answered_names;
use avatar_routing::names::vocabulary::Vocabulary;
use avatar_routing::text::folding::folded;
use std::cmp::Reverse;
use std::path::PathBuf;
use std::sync::OnceLock;

pub const RECOGNIZED_TERMS: [&str; 10] = [
    "ElevenLabs",
    "Hex",
    "NixOS",
    "chise",
    "Claude",
    "Hyprland",
    "dotfiles",
    "rebuild",
    "Lucas",
    "Lulu",
];
pub const KEYTERMS_BEFORE_MINIMUM_BILLING: usize = 100;
const ROSTER_SEPARATOR: &str = " | ";

#[derive(Debug)]
pub struct KeytermSource {
    servant_roster: Option<PathBuf>,
    vocabulary: Option<Vocabulary>,
    terms: OnceLock<Vec<String>>,
}

impl KeytermSource {
    #[must_use]
    pub const fn new(servant_roster: Option<PathBuf>, vocabulary: Option<Vocabulary>) -> Self {
        Self {
            servant_roster,
            vocabulary,
            terms: OnceLock::new(),
        }
    }

    pub fn terms(&self) -> &[String] {
        self.terms.get_or_init(|| self.load())
    }

    fn load(&self) -> Vec<String> {
        let roster = self
            .servant_roster
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok());
        keyterms(roster.as_deref(), self.vocabulary.as_ref())
    }
}

#[must_use]
pub fn keyterms(roster_text: Option<&str>, vocabulary: Option<&Vocabulary>) -> Vec<String> {
    let fixed = RECOGNIZED_TERMS.iter().map(|term| (*term).to_owned());
    let Some(text) = roster_text else {
        return fixed.collect();
    };
    let mut names = unfixed_answered_names(&servant_names(text));
    names.sort_by_key(|name| familiarity_key(name, vocabulary));
    names.truncate(KEYTERMS_BEFORE_MINIMUM_BILLING - RECOGNIZED_TERMS.len());
    fixed.chain(names).collect()
}

fn unfixed_answered_names(servants: &[String]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let answered = servants.iter().flat_map(|servant| answered_names(servant));
    for name in answered.filter(|name| !RECOGNIZED_TERMS.contains(&name.as_str())) {
        push_unless_present(&mut names, name);
    }
    names
}

fn servant_names(roster_text: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let called = roster_text.lines().map(called_servant_name);
    for name in called.filter(|name| !name.is_empty()) {
        push_unless_present(&mut names, name);
    }
    names
}

fn push_unless_present(names: &mut Vec<String>, name: String) {
    if !names.contains(&name) {
        names.push(name);
    }
}

fn called_servant_name(line: &str) -> String {
    let before_separator = line.split(ROSTER_SEPARATOR).next().unwrap_or_default();
    strip_parentheticals(before_separator).trim().to_owned()
}

fn strip_parentheticals(text: &str) -> String {
    let mut kept = String::new();
    let mut rest = text;
    while let Some((before, after)) = first_parenthetical(rest) {
        kept.push_str(before);
        rest = after;
    }
    kept + rest
}

fn first_parenthetical(text: &str) -> Option<(&str, &str)> {
    let open = text.find('(')?;
    let close = text[open..].find(')')? + open;
    Some((text[..open].trim_end(), &text[close + 1..]))
}

fn familiarity_key(name: &str, vocabulary: Option<&Vocabulary>) -> Option<Reverse<usize>> {
    let rank = vocabulary?.get(&folded(name))?;
    Some(Reverse(*rank))
}
