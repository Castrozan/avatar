use avatar_routing::hearing::wake_phrase::{
    WakeMatch, addressed_session, named_in_turn, opens_with_greeting, sessions_heard_anywhere,
    vocative_session,
};
use avatar_routing::names::session::Session;
use avatar_routing::names::vocabulary::Vocabulary;

const ORDINARY_WORDS: &str = "the and tell hear wake check build name moon night see mesa ana ontem fala tudo bem ok hey funny anyone yesterday";
const ROSTER: &str = "Ishtar=p-ishtar;Astolfo=p-astolfo;Anastasia=p-anastasia;Katsushika=p-katsushika;Mélusine=p-melusine";
const TWIN_ROSTER: &str = "Ishtar=p-ishtar;Astolfo=p-astolfo;Anastasia=p-anastasia;Katsushika=p-katsushika;Mélusine=p-melusine;Astolfo=p-astolfo-2";

const CASES: &str = r"
named | I haven't said hey or something yet. Uh, I think Astolfo, Astolfo | y | p-astolfo|I haven't said hey or something yet. Uh, I think Astolfo, Astolfo
named | Astolfo, can you take over and check the build? | y | p-astolfo|Astolfo, can you take over and check the build
named | Hey Anastasia, can you hear me? | y | p-anastasia|can you hear me
named | Astolfo... Astolfo... why did I even name it Astolfo | y | p-astolfo|Astolfo... Astolfo... why did I even name it Astolfo
named | Anastasia、聞こえる？ | y | p-anastasia|Anastasia、聞こえる？
named |  | y | -
named | Hey Mélusine, tudo bem? | y | p-melusine|tudo bem
named | Astolfo, Astolfo | n | p-astolfo|Astolfo, Astolfo
addressed | Astolfo, can you take over and check the build? | y | -
addressed | Hey Anastasia, can you hear me? | y | p-anastasia|can you hear me
addressed | Hey Astolfo | y | p-astolfo|
addressed | Hey, Anastasia ? | n | p-anastasia|
vocative | Astolfo, can you take over and check the build? | y | p-astolfo
vocative | Hokusai, tu tá aí? | y | p-katsushika
anywhere | I was telling Astolfo yesterday that the wake is broken | y | p-astolfo
anywhere | ねえシュテン、聞こえる？ | y | 
greeting | Hey Astolfo | - | true
greeting | Ishtar | - | false
greeting |  | - | false
twin | Hey Astolfo | y | -
twin | Hey Anastasia, hey Astolfo? | y | p-anastasia|hey Astolfo
named | Hey | y | -
named | Can you check the build, Astolfo? | y | p-astolfo|Can you check the build, Astolfo
vocative | Can you check the build, Astolfo? | y | p-astolfo
vocative | Hey | y | -
named | Anastasia, anyone? | y | p-anastasia|Anastasia, anyone
named | La la la, Anastasia, in the snow tonight | y | -
named | Ishtar, let's talk | y | p-ishtar|Ishtar, let's talk
";

fn ordinary_vocabulary() -> Vocabulary {
    ORDINARY_WORDS
        .split_whitespace()
        .enumerate()
        .map(|(rank, word)| (word.to_owned(), rank))
        .collect()
}

fn roster(entries: &str) -> Vec<Session> {
    entries
        .split(';')
        .filter_map(|entry| entry.split_once('='))
        .map(session_of)
        .collect()
}

fn session_of((servant, pane): (&str, &str)) -> Session {
    Session {
        servant: servant.to_owned(),
        pane: pane.to_owned(),
        host: None,
    }
}

fn wake_text(found: Option<WakeMatch>) -> String {
    found.map_or("-".to_owned(), |wake| {
        format!("{}|{}", wake.session.pane, wake.message)
    })
}

fn pane_text(found: Option<Session>) -> String {
    found.map_or("-".to_owned(), |session| session.pane)
}

fn panes_text(sessions: Vec<Session>) -> String {
    sessions
        .into_iter()
        .map(|session| session.pane)
        .collect::<Vec<_>>()
        .join(",")
}

fn outcome(kind: &str, text: &str, vocabulary: Option<&Vocabulary>, roster: &[Session]) -> String {
    match kind {
        "named" | "twin" => wake_text(named_in_turn(text, roster, vocabulary)),
        "addressed" => wake_text(addressed_session(text, roster, vocabulary)),
        "vocative" => pane_text(vocative_session(text, roster, vocabulary)),
        "anywhere" => panes_text(sessions_heard_anywhere(text, roster, vocabulary)),
        "greeting" => opens_with_greeting(text).to_string(),
        _ => format!("unknown kind {kind}"),
    }
}

fn check_row(vocabulary: &Vocabulary, row: &str) {
    let fields: Vec<&str> = row.splitn(4, '|').map(str::trim).collect();
    let entries = [ROSTER, TWIN_ROSTER][usize::from(fields[0] == "twin")];
    let with_vocabulary = (fields[2] == "y").then_some(vocabulary);
    let actual = outcome(fields[0], fields[1], with_vocabulary, &roster(entries));
    assert_eq!(actual, fields[3], "{row}");
}

#[test]
fn wake_phrase_agrees_with_python_reference() {
    let vocabulary = ordinary_vocabulary();
    CASES
        .lines()
        .filter(|line| !line.trim().is_empty())
        .for_each(|row| check_row(&vocabulary, row));
}
