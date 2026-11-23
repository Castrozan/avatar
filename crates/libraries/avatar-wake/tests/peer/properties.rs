use avatar_routing::names::session::Session;
use avatar_wake::peer::listing::sessions_everywhere;
use proptest::prelude::*;

proptest! {
    #[test]
    fn every_servant_is_heard_once_and_the_local_session_wins_for_any_listing(
        local in prop::collection::vec(session_pair(), 0..5),
        remote in prop::collection::vec(session_pair(), 0..5),
    ) {
        let merged = sessions_everywhere(&local, &remote);
        let local_servants: Vec<&str> = local.iter().map(|session| session.servant.as_str()).collect();
        prop_assert_eq!(&merged[..local.len()], local.as_slice());
        let expected_remote: Vec<&Session> = remote
            .iter()
            .filter(|session| !local_servants.contains(&session.servant.as_str()))
            .collect();
        prop_assert_eq!(merged[local.len()..].iter().collect::<Vec<_>>(), expected_remote);
    }
}

fn session_pair() -> impl Strategy<Value = Session> {
    ("Voyager|Medea|Circe|Ibaraki", "[a-z]{1,3}").prop_map(|(servant, pane)| Session {
        servant,
        pane,
        host: None,
    })
}
