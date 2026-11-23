use std::io::{self, Write};
use std::process::ExitCode;

use avatar_adapters::commands::{DoctorCommands, Programs};
use avatar_doctor::doctor::{
    Finding, LocalPageProbe, all_healthy, host_report, rendered_json, rendered_text, seat_report,
};
use clap::Args;

pub(crate) mod switch_latency;
use nix::unistd::getuid;

#[derive(Args)]
pub(crate) struct DoctorOptions {
    #[arg(long, help = "Print the findings as one JSON array")]
    json: bool,
    #[arg(
        long,
        help = "Examine the avatar seat on this host instead of this machine"
    )]
    seat_of: Option<String>,
}

fn findings(options: &DoctorOptions) -> Vec<Finding> {
    let runner = DoctorCommands::new(Programs::on_path());
    match options.seat_of.as_deref().filter(|host| !host.is_empty()) {
        Some(host) => seat_report(&runner, &LocalPageProbe::default(), host, getuid().as_raw()),
        None => host_report(&runner),
    }
}

fn rendered(findings: &[Finding], as_json: bool) -> String {
    if as_json {
        return rendered_json(findings).unwrap_or_else(|error| error.to_string());
    }
    rendered_text(findings)
}

pub(crate) fn run(options: &DoctorOptions) -> io::Result<ExitCode> {
    let findings = findings(options);
    writeln!(io::stdout().lock(), "{}", rendered(&findings, options.json))?;
    Ok(if all_healthy(&findings) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}
