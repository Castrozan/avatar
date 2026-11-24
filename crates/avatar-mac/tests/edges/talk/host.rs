use avatar_mac::edge::room::MacHostTalk;
use avatar_mac::talk::HostTalk;

use super::fake::Fake;
use super::{seat_config, strings};

const FORWARD: [&str; 7] = [
    "ssh",
    "-o",
    "BatchMode=yes",
    "-o",
    "ConnectTimeout=10",
    "elsewhere",
    "env AVATAR_CALLER_HOLDS_INPUT=1 env HERDR_PANE_ID=p1 avatar-talk say hi",
];
const HEARD: &str = "[voice] hello\n";

fn host_in(room: &str) -> MacHostTalk<Fake> {
    let mut fake = Fake::default();
    fake.environment
        .insert("AVATAR_ROOM".to_owned(), room.to_owned());
    fake.environment
        .insert("HERDR_PANE_ID".to_owned(), "p1".to_owned());
    fake.outputs.insert("ssh".to_owned(), (7, HEARD.to_owned()));
    MacHostTalk {
        boundary: fake,
        config: seat_config(""),
    }
}

#[test]
fn a_command_for_another_room_is_forwarded_over_batch_ssh_with_this_pane() {
    let mut host = host_in("elsewhere");
    assert_eq!(
        host.forwarded_from_this_machine(&strings(&["say", "hi"])),
        Some(7)
    );
    assert_eq!(host.boundary.ran, vec![strings(&FORWARD)]);
    assert_eq!(host.boundary.out, HEARD);
}

#[test]
fn nothing_is_sent_to_the_host_before_a_conversation_is_remembered_there() {
    let mut host = host_in("");
    let sent = host.forwarded_to_the_conversation_opened_elsewhere(&strings(&["follow"]));
    assert_eq!(sent, None);
    assert!(host.boundary.ran.is_empty());
}

#[test]
fn a_conversation_remembered_at_the_host_receives_the_later_commands() {
    let mut host = host_in("");
    host.remember_the_conversation_at_the_host();
    let sent = host.forwarded_to_the_conversation_opened_elsewhere(&strings(&["follow"]));
    assert_eq!(sent, Some(7));
    assert_eq!(host.boundary.ran[0][5], "chise");
    assert_eq!(host.boundary.out, HEARD);
}

#[test]
fn a_command_for_this_room_is_not_forwarded() {
    let mut host = host_in("here");
    assert_eq!(
        host.forwarded_from_this_machine(&strings(&["say", "hi"])),
        None
    );
    assert!(host.boundary.ran.is_empty());
}

#[test]
fn the_remote_talk_adds_the_live_agent_so_the_arguments_pass_through_unchanged() {
    let mut host = host_in("here");
    let arguments = strings(&["open", "--live"]);
    assert_eq!(
        host.with_the_agent_that_receives_live_tasks(&arguments),
        arguments
    );
}
