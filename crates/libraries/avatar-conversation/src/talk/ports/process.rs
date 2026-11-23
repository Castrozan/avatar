#[derive(Debug, Clone, PartialEq)]
pub struct Invocation {
    pub argv: Vec<String>,
    pub timeout_seconds: Option<f64>,
}

impl Invocation {
    #[must_use]
    pub fn new<Part: AsRef<str>>(argv: &[Part], timeout_seconds: Option<f64>) -> Self {
        Self {
            argv: argv.iter().map(|part| part.as_ref().to_owned()).collect(),
            timeout_seconds,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    pub status: i32,
    pub stdout: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessFailure {
    Spawn(String),
    TimedOut,
}

pub trait Subprocess {
    fn run(&mut self, invocation: &Invocation) -> Result<ProcessOutput, ProcessFailure>;
}
