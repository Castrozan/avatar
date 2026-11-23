use avatar_adapters::commands::TalkProcess;
use avatar_conversation::talk::ports::{Invocation, ProcessFailure, Subprocess};

use crate::tools::{TestResult, Tools};

type Outcome = Result<(i32, String), ProcessFailure>;

fn invoke(tools: &Tools, words: &[&str], timeout: Option<f64>) -> Outcome {
    let mut process = TalkProcess::new(tools.programs());
    let output = process.run(&Invocation::new(words, timeout))?;
    Ok((output.status, output.stdout))
}

#[test]
fn hyprctl_gets_its_argv_and_answers_stdout_and_status() -> TestResult {
    let tools = Tools::with(&[("hyprctl", "echo \"$@\"; exit 4")])?;
    let outcome = invoke(
        &tools,
        &["hyprctl", "--instance", "0", "-j", "clients"],
        None,
    );
    assert_eq!(outcome, Ok((4, "--instance 0 -j clients\n".to_owned())));
    Ok(())
}

#[test]
fn discord_voice_launcher_replies_are_returned_whole() -> TestResult {
    let tools = Tools::with(&[("discord-voice", "echo '{\"joined\": true}'")])?;
    let outcome = invoke(&tools, &["discord-voice", "status"], Some(5.0));
    assert_eq!(outcome, Ok((0, "{\"joined\": true}\n".to_owned())));
    Ok(())
}

#[test]
fn a_missing_program_fails_like_python_reports_it() -> TestResult {
    let tools = Tools::with(&[])?;
    let text = "[Errno 2] No such file or directory: 'absent'".to_owned();
    let outcome = invoke(&tools, &["absent"], None);
    assert_eq!(outcome, Err(ProcessFailure::Spawn(text)));
    Ok(())
}

#[test]
fn a_command_over_its_timeout_fails_as_timed_out() -> TestResult {
    let tools = Tools::with(&[("slow", "exec sleep 5")])?;
    let outcome = invoke(&tools, &["slow"], Some(0.2));
    assert_eq!(outcome, Err(ProcessFailure::TimedOut));
    Ok(())
}
