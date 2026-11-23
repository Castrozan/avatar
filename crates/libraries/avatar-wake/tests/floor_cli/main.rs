use avatar_floor::addressee::Addressee;
use avatar_floor::floor::RemoteAgent;
use avatar_wake::floor_cli::{
    ClaimOutcome, NO_SERVANT_MESSAGE, pane_claim_outcome, refusal_message, speaker_claim_outcome,
};
use avatar_wake::panes::{LivePane, live_sessions};

fn addressee(pane: &str, servant: &str, host: Option<&str>) -> Addressee {
    Addressee {
        pane: pane.to_owned(),
        servant: servant.to_owned(),
        heard_through: 0,
        host: host.map(str::to_owned),
    }
}

fn pane(pane_id: &str, servant: Option<&str>) -> LivePane {
    LivePane {
        pane_id: pane_id.to_owned(),
        servant: servant.map(str::to_owned),
    }
}

fn kira_speaker() -> RemoteAgent {
    RemoteAgent {
        pane: "w1:p2".to_owned(),
        host: Some("kira".to_owned()),
    }
}

fn speaks(servant: &str) -> ClaimOutcome {
    ClaimOutcome::Speaks {
        servant: Some(servant.to_owned()),
    }
}

fn refused(servant: &str) -> ClaimOutcome {
    ClaimOutcome::Refused {
        message: refusal_message(servant),
    }
}

#[test]
fn only_panes_with_a_servant_name_are_live_sessions() {
    let panes = [
        pane("p1", Some("Voyager")),
        pane("p2", Some("")),
        pane("p3", None),
    ];
    let sessions = live_sessions(&panes);
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].servant, "Voyager");
    assert_eq!(sessions[0].pane, "p1");
    assert_eq!(sessions[0].host, None);
}

#[test]
fn the_refusal_names_the_holder() {
    assert_eq!(
        refusal_message("Medea"),
        "Medea holds the avatar's floor: stay silent until the room names you."
    );
}

#[test]
fn the_pane_without_a_name_is_explained() {
    assert!(NO_SERVANT_MESSAGE.starts_with("The pane has no Servant name"));
}

#[test]
fn a_pane_claim_is_refused_for_another_pane_holder() {
    let holder = addressee("p9", "Circe", None);
    let outcome = pane_claim_outcome(Some(&holder), "p1", || Some("Voyager".to_owned()));
    assert_eq!(outcome, refused("Circe"));
}

#[test]
fn a_pane_claim_by_the_holder_speaks_as_the_holder() {
    let holder = addressee("p1", "Voyager", None);
    assert_eq!(
        pane_claim_outcome(Some(&holder), "p1", || None),
        speaks("Voyager")
    );
}

#[test]
fn without_a_holder_the_pane_servant_is_used() {
    let outcome = pane_claim_outcome(None, "p1", || Some("Voyager".to_owned()));
    assert_eq!(outcome, speaks("Voyager"));
}

#[test]
fn a_speaker_on_the_holding_pane_and_host_speaks() {
    let same = addressee("w1:p2", "Medea", Some("kira"));
    let outcome = speaker_claim_outcome(Some(&same), &kira_speaker(), Some("Medea".to_owned()));
    assert_eq!(outcome, speaks("Medea"));
}

#[test]
fn a_speaker_is_refused_when_another_host_holds_the_floor() {
    let other_host = addressee("w1:p2", "Medea", None);
    let outcome = speaker_claim_outcome(Some(&other_host), &kira_speaker(), None);
    assert_eq!(outcome, refused("Medea"));
}
