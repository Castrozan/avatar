use avatar_seat::edges::{CommandOutcome, EdgeFailure};
use serde_json::Value;

pub(super) enum Raise {
    Timeout,
    Os(String),
}

#[derive(Default)]
pub(super) struct RunScript {
    code: i32,
    stdout: String,
    raise: Option<Raise>,
}

fn field<'entry>(entry: &'entry Value, name: &str, position: usize) -> Option<&'entry Value> {
    entry.get(name).or_else(|| entry.get(position))
}

fn raise_of(name: Option<&str>, os_message: &str) -> Option<Raise> {
    let timeout = name.filter(|kind| *kind == "timeout");
    let os = name.map(|_| Raise::Os(os_message.to_owned()));
    timeout.map(|_| Raise::Timeout).or(os)
}

impl RunScript {
    pub(super) fn from_value(entry: &Value, os_message: &str) -> Self {
        let text = |name, position| field(entry, name, position).and_then(Value::as_str);
        let code = field(entry, "code", 0).and_then(Value::as_i64);
        Self {
            code: code
                .and_then(|value| i32::try_from(value).ok())
                .unwrap_or(0),
            stdout: text("stdout", 1).unwrap_or_default().to_owned(),
            raise: raise_of(text("raise", 2), os_message),
        }
    }

    pub(super) fn into_outcome(
        self,
        command: &[String],
        seconds: u64,
    ) -> Result<CommandOutcome, EdgeFailure> {
        match self.raise {
            Some(raise) => Err(raise.failure(command, seconds)),
            None => Ok(CommandOutcome {
                code: self.code,
                stdout: self.stdout,
            }),
        }
    }
}

impl Raise {
    fn failure(self, command: &[String], seconds: u64) -> EdgeFailure {
        match self {
            Self::Timeout => EdgeFailure::TimedOut {
                command: command.to_vec(),
                seconds,
            },
            Self::Os(message) => EdgeFailure::Os(message),
        }
    }
}
