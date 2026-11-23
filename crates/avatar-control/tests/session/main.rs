use std::error::Error;

type TestResult = Result<(), Box<dyn Error>>;

fn ensure(condition: bool, message: impl FnOnce() -> String) -> TestResult {
    if condition {
        Ok(())
    } else {
        Err(message().into())
    }
}

mod contracts;
mod document;
mod observation;
mod origins;
mod replay;
