use avatar_mac::failure::Failure;
use avatar_mac::talk::{HostTalk, LiveOpening, LiveStageHooks, TalkRequest, carry_out};
use proptest::prelude::*;
use serde_json::{Value, json};

use crate::world::World;

#[derive(Default)]
struct Reach {
    host_answers: i32,
    there_answers: Option<i32>,
    remembered: bool,
    carried: bool,
}

impl HostTalk for Reach {
    fn forwarded_from_this_machine(&mut self, _arguments: &[String]) -> Option<i32> {
        None
    }

    fn forwarded_to_the_conversation_opened_elsewhere(
        &mut self,
        _arguments: &[String],
    ) -> Option<i32> {
        self.there_answers
    }

    fn remember_the_conversation_at_the_host(&mut self) {
        self.remembered = true;
    }

    fn with_the_agent_that_receives_live_tasks(&mut self, arguments: &[String]) -> Vec<String> {
        arguments.to_vec()
    }

    fn carry_out_at_host(&mut self, _arguments: &[String]) -> Result<i32, Failure> {
        self.carried = true;
        Ok(self.host_answers)
    }
}

struct NoLive;

impl LiveOpening for NoLive {
    fn open_live_conversation(
        &mut self,
        _request: &TalkRequest,
        _hooks: &mut dyn LiveStageHooks,
    ) -> Result<(), Failure> {
        Ok(())
    }
}

fn room(listening: bool) -> World {
    let request = json!({"command": "status", "text": "", "emotion": "", "wait": 0});
    let seat = json!({"reachable": false, "listening": listening});
    World::new(&json!({"phase": "closed", "request": request, "seat": seat}))
}

fn carried(host: &mut Reach, listening: bool, arguments: &[&str]) -> Value {
    let words: Vec<String> = arguments.iter().map(|word| (*word).to_owned()).collect();
    let mut world = room(listening);
    let result = carry_out(&mut world, host, &mut NoLive, &words);
    json!([result.ok(), world.stdout])
}

#[test]
fn a_command_goes_where_this_mac_opened_the_conversation_instead_of_running_here() {
    let mut host = Reach {
        there_answers: Some(5),
        ..Reach::default()
    };
    assert_eq!(carried(&mut host, false, &["status"]), json!([5, ""]));
    assert!(!host.carried);
}

#[test]
fn a_seat_listening_here_still_sends_the_command_to_the_host() {
    let mut host = Reach {
        there_answers: Some(5),
        host_answers: 2,
        ..Reach::default()
    };
    assert_eq!(carried(&mut host, true, &["status"]), json!([2, ""]));
    assert!(host.carried);
}

proptest! {
    #[test]
    fn the_host_is_remembered_only_after_it_opened_a_seat_that_is_not_this_mac(
        seat in "discord|room|remote",
        host_answers in 0..3_i32,
    ) {
        let mut host = Reach { host_answers, ..Reach::default() };
        carried(&mut host, false, &["open", "--seat", &seat]);
        prop_assert_eq!(host.remembered, seat != "remote" && host_answers == 0);
    }
}
