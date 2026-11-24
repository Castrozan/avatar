use std::io;
use std::thread;

use avatar_conversation::room_reach::CALLER_HOLDS_INPUT_VARIABLE;
use nix::sys::signal::{Signal, raise};

const LISTENING_VERBS: [&str; 3] = ["hear", "ask", "follow"];

pub(crate) fn end_a_listening_command_with_its_caller(arguments: &[String]) {
    let listens = arguments
        .first()
        .is_some_and(|verb| LISTENING_VERBS.contains(&verb.as_str()));
    let held = std::env::var_os(CALLER_HOLDS_INPUT_VARIABLE).is_some();
    if listens && held {
        thread::spawn(end_when_the_input_closes);
    }
}

fn end_when_the_input_closes() {
    let _ = io::copy(&mut io::stdin().lock(), &mut io::sink());
    let _ = raise(Signal::SIGTERM);
}
