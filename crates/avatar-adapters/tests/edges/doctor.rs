use std::time::Duration;

use avatar_adapters::commands::DoctorCommands;
use avatar_doctor::doctor::CommandRunner;

use crate::fakes::{TestResult, Tools};

fn text_of_error(runner: &DoctorCommands, program: &str) -> Option<String> {
    let failure = runner.run(&[program], &[]).err();
    failure.map(|error| error.to_string())
}

#[test]
fn doctor_commands_pass_stdin_and_capture_both_streams() -> TestResult {
    let tools = Tools::with(&[("tool", "cat; echo err >&2; echo \"$1\"; exit 3")])?;
    let output = DoctorCommands::new(tools.programs()).run(&["tool", "arg"], b"in\n")?;
    assert!(!output.succeeded);
    assert_eq!(output.stdout, b"in\narg\n");
    assert_eq!(output.last_stderr_line(), "err");
    Ok(())
}

#[test]
fn doctor_commands_time_out_with_the_python_error_name() -> TestResult {
    let tools = Tools::with(&[("slow", "sleep 5")])?;
    let runner = DoctorCommands::new(tools.programs()).with_timeout(Duration::from_millis(200));
    let text = text_of_error(&runner, "slow");
    assert!(text.is_some_and(|text| text.starts_with("TimeoutExpired")));
    Ok(())
}

#[test]
fn doctor_commands_report_missing_programs_as_os_errors() -> TestResult {
    let tools = Tools::with(&[])?;
    let text = text_of_error(&DoctorCommands::new(tools.programs()), "absent");
    assert!(text.is_some_and(|text| text.starts_with("OSError")));
    Ok(())
}
