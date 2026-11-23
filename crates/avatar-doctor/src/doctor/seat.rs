use super::command::{CommandOutput, CommandRunner};
use super::failure::DoctorError;
use super::finding::Finding;
use super::host::transcriber_answers;
use super::mac::mac_agents_loaded;
use super::page::PageProbe;
use super::wake::wake_microphone_hears;

const REACH: [&str; 5] = ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=5"];

pub(crate) fn seat_findings(
    runner: &impl CommandRunner,
    probe: &impl PageProbe,
    host: &str,
    user_id: u32,
) -> Result<Vec<Finding>, DoctorError> {
    let reach = host_reachable(runner, host)?;
    if reach.healthy {
        return reachable_seat(runner, probe, host, user_id, reach);
    }
    unreachable_seat(runner, user_id, reach)
}

fn reachable_seat(
    runner: &impl CommandRunner,
    probe: &impl PageProbe,
    host: &str,
    user_id: u32,
    reach: Finding,
) -> Result<Vec<Finding>, DoctorError> {
    let mut findings = vec![reach];
    findings.extend(findings_reported_by(runner, host)?);
    findings.extend(mac_agents_loaded(runner, user_id)?);
    findings.push(wake_microphone_hears(runner, probe)?);
    findings.push(transcriber_answers(runner)?);
    Ok(findings)
}

fn unreachable_seat(
    runner: &impl CommandRunner,
    user_id: u32,
    reach: Finding,
) -> Result<Vec<Finding>, DoctorError> {
    let mut findings = vec![reach];
    findings.extend(mac_agents_loaded(runner, user_id)?);
    findings.push(transcriber_answers(runner)?);
    Ok(findings)
}

fn reach_command<'a>(host: &'a str, rest: &[&'a str]) -> Vec<&'a str> {
    let mut command = REACH.to_vec();
    command.push(host);
    command.extend_from_slice(rest);
    command
}

fn host_reachable(runner: &impl CommandRunner, host: &str) -> Result<Finding, DoctorError> {
    let output = runner.run(&reach_command(host, &["true"]), &[])?;
    let detail = reach_detail(&output);
    Ok(Finding::new(
        format!("reach {host}"),
        output.succeeded,
        detail,
    ))
}

fn reach_detail(output: &CommandOutput) -> String {
    if output.succeeded {
        "reachable".to_owned()
    } else {
        output.last_stderr_line()
    }
}

fn findings_reported_by(
    runner: &impl CommandRunner,
    host: &str,
) -> Result<Vec<Finding>, DoctorError> {
    let output = runner.run(&reach_command(host, &["avatar-doctor", "--json"]), &[])?;
    reported_findings(&output.stdout, host)
}

fn reported_findings(stdout: &[u8], host: &str) -> Result<Vec<Finding>, DoctorError> {
    let reported: Vec<Finding> = serde_json::from_slice(stdout)?;
    Ok(reported
        .into_iter()
        .map(|finding| prefixed(host, finding))
        .collect())
}

fn prefixed(host: &str, finding: Finding) -> Finding {
    Finding {
        name: format!("{host}: {}", finding.name),
        ..finding
    }
}
