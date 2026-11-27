use avatar_conversation::talk::cli::arguments::parse_talk_arguments;
use avatar_listening::conversation_state::{ConversationCommand, ListenerPurpose};
use avatar_mac::edge::room::MacLiveOpening;
use avatar_mac::failure::Failure;
use avatar_mac::talk::{LiveOpening, LiveStageHooks, TalkRequest};

use super::fake::Fake;
use super::strings;

#[derive(Default)]
pub(super) struct Hooks(Vec<String>);

impl LiveStageHooks for Hooks {
    fn bring_up(&mut self, seat: &str) -> Result<(), Failure> {
        self.0.push(format!("bring_up {seat}"));
        Ok(())
    }

    fn wait_until_seated(&mut self) -> Result<(), Failure> {
        self.0.push("wait_until_seated".to_owned());
        Ok(())
    }

    fn close(&mut self) -> Result<(), Failure> {
        self.0.push("close".to_owned());
        Ok(())
    }
}

fn request() -> TalkRequest {
    let words = strings(&["open", "--live", "--character", "Mara"]);
    let open = parse_talk_arguments(&words)
        .map(|talk| talk.open())
        .unwrap_or_default();
    let mut request = TalkRequest::new(ConversationCommand::OpenLive);
    request.character = open.character;
    request
}

pub(super) fn live_with(pane: Option<&str>, granting: bool) -> MacLiveOpening<Fake> {
    let mut fake = Fake {
        granting,
        ..Fake::default()
    };
    fake.active.insert("avatar-conversation.target".to_owned());
    fake.active.insert("avatar-live.service".to_owned());
    fake.environment
        .extend(pane.map(|name| ("HERDR_PANE_ID".to_owned(), name.to_owned())));
    MacLiveOpening { boundary: fake }
}

fn opened(pane: Option<&str>, granting: bool) -> (MacLiveOpening<Fake>, Hooks, Option<Failure>) {
    let mut live = live_with(pane, granting);
    let mut hooks = Hooks::default();
    let failure = live.open_live_conversation(&request(), &mut hooks).err();
    (live, hooks, failure)
}

#[test]
fn the_live_voice_needs_a_pane_and_touches_nothing_without_one() {
    let (_live, hooks, failure) = opened(None, true);
    assert!(failure.is_some_and(|found| found.message.contains("herdr pane")));
    assert!(hooks.0.is_empty());
}

#[test]
fn the_live_voice_seats_the_listener_through_the_stage_hooks() {
    let (live, hooks, failure) = opened(Some("p1"), true);
    assert!(failure.is_none());
    assert_eq!(hooks.0, strings(&["bring_up room", "wait_until_seated"]));
    assert_eq!(live.boundary.begun[0].purpose, ListenerPurpose::Live);
    assert_eq!(live.boundary.begun[0].human_source, "AvatarRoomMicrophone");
}

#[test]
fn the_live_voice_shows_the_character_and_claims_the_floor() {
    let (live, _hooks, _failure) = opened(Some("p1"), true);
    let character = strings(&["avatar-command", "character", "Mara"]);
    assert!(live.boundary.ran.contains(&character));
    assert!(
        live.boundary
            .ran
            .contains(&strings(&["avatar-claim-floor"]))
    );
}

#[test]
fn the_live_voice_reports_the_status_of_the_live_conversation() {
    let (live, _hooks, _failure) = opened(Some("p1"), true);
    let status: serde_json::Value =
        serde_json::from_str(live.boundary.out.trim()).unwrap_or_default();
    assert_eq!(status["live"], true);
}

#[test]
fn a_pane_without_a_servant_leaves_the_conversation_it_opened() {
    let (_live, hooks, failure) = opened(Some("p1"), false);
    assert!(failure.is_some_and(|found| found.message.contains("Servant name")));
    assert_eq!(hooks.0.last().map(String::as_str), Some("close"));
}
