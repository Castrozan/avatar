use crate::talk::failure::TalkFailure;
use crate::talk::ports::{Invocation, ProcessOutput, Subprocess};

pub fn run_unchecked<Part: AsRef<str>>(
    process: &mut impl Subprocess,
    argv: &[Part],
    timeout_seconds: Option<f64>,
) -> Result<ProcessOutput, TalkFailure> {
    let invocation = Invocation::new(argv, timeout_seconds);
    process
        .run(&invocation)
        .map_err(|failure| TalkFailure::from_process(failure, &invocation))
}

pub fn run_checked<Part: AsRef<str>>(
    process: &mut impl Subprocess,
    argv: &[Part],
    timeout_seconds: Option<f64>,
) -> Result<ProcessOutput, TalkFailure> {
    let output = run_unchecked(process, argv, timeout_seconds)?;
    check_status(argv, output.status)?;
    Ok(output)
}

pub fn check_status<Part: AsRef<str>>(argv: &[Part], status: i32) -> Result<(), TalkFailure> {
    if status == 0 {
        return Ok(());
    }
    let parts: Vec<String> = argv.iter().map(|part| part.as_ref().to_owned()).collect();
    Err(TalkFailure::non_zero_exit(&parts, status))
}
