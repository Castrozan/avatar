use serde::{Deserialize, Serialize};

use super::failure::DoctorError;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub name: String,
    pub healthy: bool,
    pub detail: String,
}

impl Finding {
    #[must_use]
    pub fn new(name: impl Into<String>, healthy: bool, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            healthy,
            detail: detail.into(),
        }
    }
}

#[must_use]
pub fn examined(examination: Result<Vec<Finding>, DoctorError>) -> Vec<Finding> {
    examination.unwrap_or_else(|error| vec![Finding::new("doctor", false, error.to_string())])
}

#[must_use]
pub fn all_healthy(findings: &[Finding]) -> bool {
    findings.iter().all(|finding| finding.healthy)
}

#[must_use]
pub fn rendered_text(findings: &[Finding]) -> String {
    findings
        .iter()
        .map(rendered_line)
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn rendered_json(findings: &[Finding]) -> Result<String, DoctorError> {
    Ok(serde_json::to_string(findings)?)
}

fn rendered_line(finding: &Finding) -> String {
    let status = if finding.healthy { "ok  " } else { "FAIL" };
    format!("{status} {}: {}", finding.name, finding.detail)
}
