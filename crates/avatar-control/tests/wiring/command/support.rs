use std::error::Error;
use std::fmt::Display;

pub(crate) type TestResult = Result<(), Box<dyn Error>>;

pub(crate) fn check(condition: bool, detail: impl Display) -> TestResult {
    if condition {
        return Ok(());
    }
    Err(detail.to_string().into())
}
