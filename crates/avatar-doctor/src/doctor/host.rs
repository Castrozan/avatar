use super::command::CommandRunner;
use super::failure::DoctorError;
use super::finding::Finding;

const HOST_UNITS: [&str; 2] = ["hex.service", "avatar-wake.service"];
const SILENCE: [u8; 16_000] = [0; 16_000];

pub(crate) fn host_findings(runner: &impl CommandRunner) -> Result<Vec<Finding>, DoctorError> {
    let mut findings = Vec::new();
    for unit in HOST_UNITS {
        findings.push(unit_active(runner, unit)?);
    }
    findings.push(transcriber_answers(runner)?);
    Ok(findings)
}

pub(crate) fn unit_active(runner: &impl CommandRunner, unit: &str) -> Result<Finding, DoctorError> {
    let output = runner.run(&["systemctl", "--user", "is-active", unit], &[])?;
    let state = output.last_stdout_line();
    let healthy = state == "active";
    let detail = if state.is_empty() {
        "unknown".to_owned()
    } else {
        state
    };
    Ok(Finding::new(unit, healthy, detail))
}

pub(crate) fn transcriber_answers(runner: &impl CommandRunner) -> Result<Finding, DoctorError> {
    let output = runner.run(&["avatar-transcribe"], &SILENCE)?;
    let detail = if output.succeeded {
        "answers".to_owned()
    } else {
        output.last_stderr_line()
    };
    Ok(Finding::new("transcription", output.succeeded, detail))
}
