pub trait CommandRunner {
    fn stdout_of(&self, command: &[String], timeout_seconds: u64) -> Option<String>;
}

#[must_use]
pub fn ssh_command(host: &str, remote: &[&str]) -> Vec<String> {
    ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=5", host]
        .into_iter()
        .chain(remote.iter().copied())
        .map(str::to_owned)
        .collect()
}
