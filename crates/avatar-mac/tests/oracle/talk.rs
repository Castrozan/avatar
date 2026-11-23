use avatar_mac::failure::Failure;
use avatar_mac::talk::{HostTalk, LiveOpening, LiveStageHooks, TalkRequest, carry_out, run};
use serde_json::{Value, json};

use crate::stage::{compact, outcome};
use crate::support::{assert_same, each, parse, strings};
use crate::world::{Log, World};

struct Host(Log, Value);

impl HostTalk for Host {
    fn forwarded_from_this_machine(&mut self, arguments: &[String]) -> Option<i32> {
        self.0.push(json!(["forwarded", arguments]));
        self.1["forward"]
            .as_i64()
            .and_then(|code| i32::try_from(code).ok())
    }

    fn with_the_agent_that_receives_live_tasks(&mut self, arguments: &[String]) -> Vec<String> {
        self.0.push(json!(["with_agent", arguments]));
        [arguments, &["+agent".to_owned()]].concat()
    }

    fn carry_out_at_host(&mut self, arguments: &[String]) -> Result<i32, Failure> {
        self.0.push(json!(["host_carry_out", arguments]));
        let code = self.1["host_code"].as_i64();
        Ok(code.and_then(|code| i32::try_from(code).ok()).unwrap_or(0))
    }
}

struct Live(Log, Vec<String>);

impl LiveOpening for Live {
    fn open_live_conversation(
        &mut self,
        _request: &TalkRequest,
        hooks: &mut dyn LiveStageHooks,
    ) -> Result<(), Failure> {
        self.0.push(json!(["open_live"]));
        self.1.iter().try_for_each(|hook| match hook.as_str() {
            "bring_up" => hooks.bring_up("room"),
            "wait_until_seated" => hooks.wait_until_seated(),
            _ => hooks.close(),
        })
    }
}

fn edges(world: &World) -> (Host, Live) {
    let live = Live(world.log.clone(), strings(&world.config["live_script"]));
    (Host(world.log.clone(), world.config.clone()), live)
}

fn code_outcome(result: &Result<i32, Failure>) -> Value {
    match result {
        Ok(code) => json!(["code", code]),
        Err(failure) => outcome(&Err(failure.clone())),
    }
}

fn check_dispatch(case: &Value) {
    let mut world = World::new(&case["world"]);
    let (mut host, mut live) = edges(&world);
    let arguments = strings(&case["world"]["args"]);
    let result = carry_out(&mut world, &mut host, &mut live, &arguments);
    assert_eq!(code_outcome(&result), case["result"], "{case}");
    assert_same(&json!(compact(&world.log.entries())), &case["log"]);
}

fn check_failure_exit(case: &Value) {
    let Some(message) = case["result"][2].as_str() else {
        return;
    };
    let mut world = World::new(&case["world"]);
    let (mut host, mut live) = edges(&world);
    let arguments = strings(&case["world"]["args"]);
    assert_eq!(run(&mut world, &mut host, &mut live, &arguments), 1);
    assert_eq!(world.log.entries().pop(), Some(json!(["stderr", message])));
}

#[test]
fn the_talk_dispatch_behaves_as_python_mac_talk_in_every_world() {
    each(&parse(include_str!("fixtures/talk.json")), check_dispatch);
}

#[test]
fn a_failure_is_printed_and_exits_one() {
    each(
        &parse(include_str!("fixtures/talk.json")),
        check_failure_exit,
    );
}
