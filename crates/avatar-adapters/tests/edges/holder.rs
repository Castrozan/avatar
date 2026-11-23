use avatar_adapters::commands::HolderCommands;
use avatar_adapters::holder::{LineReporter, StopSignal, StoppableClock};
use avatar_holder::floor_holder::{FloorClock, HoldingInterruption, Reporter};
use avatar_holder::holder_cli::CommandRunner;

use crate::fakes::{TestResult, Tools, words};

fn failure_message(runner: &mut HolderCommands, program: &str, seconds: u64) -> Option<String> {
    let failure = runner.run_command(&words(&[program]), seconds).err();
    failure.map(|failure| failure.message().to_owned())
}

#[test]
fn holder_commands_succeed_on_zero_exit() -> TestResult {
    let tools = Tools::with(&[("good", "exit 0")])?;
    let mut runner = HolderCommands::new(tools.programs());
    assert!(runner.run_command(&words(&["good"]), 5).is_ok());
    Ok(())
}

#[test]
fn holder_commands_describe_each_kind_of_failure() -> TestResult {
    let tools = Tools::with(&[("bad", "exit 4"), ("slow", "sleep 5")])?;
    let mut runner = HolderCommands::new(tools.programs());
    let refused = failure_message(&mut runner, "bad", 5);
    assert!(refused.is_some_and(|text| text.contains("exit status 4")));
    let slow = failure_message(&mut runner, "slow", 1);
    assert!(slow.is_some_and(|text| text.contains("timed out after 1")));
    assert!(failure_message(&mut runner, "absent", 1).is_some());
    Ok(())
}

#[test]
fn reporter_accepts_messages() {
    let mut reporter = LineReporter::new(Vec::new());
    reporter.report("first");
    reporter.report("second");
}

#[test]
fn clock_pauses_fully_until_stopped() {
    let stop = StopSignal::default();
    let mut clock = StoppableClock::new(stop.clone());
    let before = clock.now_seconds();
    assert_eq!(clock.pause(0.1), Ok(()));
    assert!(clock.now_seconds() - before >= 0.1);
    stop.raise();
    assert_eq!(clock.pause(5.0), Err(HoldingInterruption::Stopped));
}
