use avatar_routing::names::session::Session;
use avatar_routing::names::spoken_names::called_name;
use avatar_wake::addressing::by_called_name;
use proptest::prelude::*;

const SERVANTS: [&str; 5] = ["Voyager", "Medea", "Circe Ninety", "Ibaraki", "Sasaki"];

fn session_strategy() -> impl Strategy<Value = Session> {
    (0..SERVANTS.len(), "[a-z:0-9]{1,4}").prop_map(|(index, pane)| Session {
        servant: SERVANTS[index].to_owned(),
        pane,
        host: None,
    })
}

proptest! {
    #[test]
    fn each_called_name_appears_once_and_keeps_the_last_session(
        sessions in prop::collection::vec(session_strategy(), 0..12)
    ) {
        let named = by_called_name(&sessions);
        let mut names: Vec<&String> = named.iter().map(|(name, _)| name).collect();
        names.sort();
        names.dedup();
        prop_assert_eq!(names.len(), named.len());
        for (name, session) in &named {
            let last = sessions.iter().rev().find(|candidate| called_name(&candidate.servant).as_ref() == Some(name));
            prop_assert_eq!(Some(session), last);
        }
    }
}
