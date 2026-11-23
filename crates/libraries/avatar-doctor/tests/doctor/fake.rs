use std::cell::RefCell;
use std::io;

use avatar_doctor::doctor::{CommandOutput, CommandRunner, DoctorError, PageProbe};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum Reply {
    Raised {
        raise: String,
    },
    Answered {
        rc: i32,
        stdout: String,
        stderr: String,
    },
}

impl Reply {
    fn respond(&self) -> Result<CommandOutput, DoctorError> {
        match self {
            Self::Raised { raise } => Err(raised_error(raise).into()),
            Self::Answered { rc, stdout, stderr } => Ok(CommandOutput {
                succeeded: *rc == 0,
                stdout: stdout.clone().into_bytes(),
                stderr: stderr.clone().into_bytes(),
            }),
        }
    }
}

fn raised_error(kind: &str) -> io::Error {
    match kind {
        "timeout" => io::Error::from(io::ErrorKind::TimedOut),
        _ => io::Error::from(io::ErrorKind::NotFound),
    }
}

pub(crate) fn answered(rc: i32, stdout: &str, stderr: &str) -> Reply {
    Reply::Answered {
        rc,
        stdout: stdout.to_owned(),
        stderr: stderr.to_owned(),
    }
}

pub(crate) fn answer(fragment: &str, reply: Reply) -> (String, Reply) {
    (fragment.to_owned(), reply)
}

#[derive(Default)]
pub(crate) struct FakeRunner {
    answers: Vec<(String, Reply)>,
    asked: RefCell<Vec<String>>,
}

impl FakeRunner {
    pub(crate) fn new(answers: Vec<(String, Reply)>) -> Self {
        Self {
            answers,
            asked: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn was_asked(&self, fragment: &str) -> bool {
        self.asked
            .borrow()
            .iter()
            .any(|command| command.contains(fragment))
    }
}

impl CommandRunner for FakeRunner {
    fn run(&self, command: &[&str], _stdin: &[u8]) -> Result<CommandOutput, DoctorError> {
        let joined = command.join(" ");
        self.asked.borrow_mut().push(joined.clone());
        match self
            .answers
            .iter()
            .find(|(fragment, _)| joined.contains(fragment.as_str()))
        {
            Some((_, reply)) => reply.respond(),
            None => answered(0, "", "").respond(),
        }
    }
}

pub(crate) struct FakeProbe {
    pub(crate) serves: bool,
}

impl PageProbe for FakeProbe {
    fn serves(&self, _port: u16) -> bool {
        self.serves
    }
}
