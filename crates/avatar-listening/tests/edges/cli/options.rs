use super::world::Report;
use crate::edges::helpers::{Failure, event_rows, strings};
use crate::support::{Outcome, text};
use serde_json::Value;

fn parsed_lines(lines: &[String]) -> Result<Vec<Value>, Failure> {
    lines
        .iter()
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}

impl Report {
    pub(super) fn check(&self, case: &Value) -> Outcome {
        self.check_failure(case)?;
        assert_eq!(
            parsed_lines(&self.lines)?,
            parsed_lines(&strings(&case["stdout"])?)?,
            "{case}"
        );
        assert_eq!(self.events, event_rows(&case["events"])?, "{case}");
        let configuration = self.configuration.clone().unwrap_or(Value::Null);
        assert_eq!(configuration, case["configuration"], "{case}");
        Ok(())
    }

    fn check_failure(&self, case: &Value) -> Outcome {
        let failure = self.failure.as_ref().map(ToString::to_string);
        assert_eq!(failure.is_some(), case["code"] == 1, "{case}");
        assert_eq!(
            failure.unwrap_or_default(),
            text(&case["stderr"])?,
            "{case}"
        );
        Ok(())
    }
}
