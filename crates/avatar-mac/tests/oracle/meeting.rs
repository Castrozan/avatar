use avatar_mac::failure::Failure;
use avatar_mac::meeting::{
    ListenCommand, MeetingRoom, SeatHealth, carry_out_verb, controls, end, listen_command,
    listen_to_the_call, start, stop,
};
use serde_json::{Value, json};

use crate::stage::{compact, outcome};
use crate::support::{assert_same, each, parse, strings};
use crate::world::{World, scripted_health};

impl MeetingRoom for World {
    fn seat_health(&mut self) -> Option<SeatHealth> {
        scripted_health(self)
    }

    fn no_page_message(&self) -> String {
        "The seat page did not attach; check that Google Chrome for Testing may use the microphone"
            .to_owned()
    }

    fn open_controls_window(&mut self, page: &str, port: u16) -> Result<(), Failure> {
        self.log.push(json!(["open_window", page, port]));
        self.fails("open_window")
    }
}

fn run(world: &mut World, operation: &Value) -> Result<(), Failure> {
    match operation[0].as_str().unwrap_or_default() {
        "start" => start(world),
        "stop" => stop(world),
        "controls" => controls(world),
        "end" => end(world),
        _ => listen_to_the_call(world, operation[1].as_str()),
    }
}

fn finish(world: &World, result: &Result<(), Failure>, case: &Value) {
    world.log.push(json!(["stdout", world.stdout]));
    assert_eq!(outcome(result), case["result"], "{case}");
    assert_same(&json!(compact(&world.log.entries())), &case["log"]);
}

fn check_operation(case: &Value) {
    let mut world = World::new(&case["world"]);
    let result = run(&mut world, &case["op"]);
    finish(&world, &result, case);
}

fn check_verb(case: &Value) {
    let mut world = World::new(&case["world"]);
    let result = carry_out_verb(&mut world, &strings(&case["argv"])).map(drop);
    finish(&world, &result, case);
}

fn listened(argv: &[String]) -> Value {
    match listen_command(argv) {
        ListenCommand::Delegate => json!(["delegate"]),
        ListenCommand::Start { language } => json!(["start", language]),
        ListenCommand::Exit(code) => json!(["exit", code]),
    }
}

fn check_listen(case: &Value) {
    assert_eq!(
        listened(&strings(&case["argv"])),
        case["result"],
        "{}",
        case["argv"]
    );
}

fn fixture() -> Value {
    parse(include_str!("fixtures/meeting.json"))
}

#[test]
fn the_meeting_behaves_as_the_python_meeting_in_every_world() {
    each(&fixture()["ops"], check_operation);
}

#[test]
fn the_meeting_verbs_accept_one_known_verb_and_nothing_else() {
    each(&fixture()["verbs"], check_verb);
}

#[test]
fn avatar_listen_start_reads_its_language_as_argparse_does() {
    each(&fixture()["listen"], check_listen);
}
