use super::USAGE;
use super::ports::MeetingRoom;
use super::steps::{controls, start, stop};
use crate::failure::Failure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListenCommand {
    Delegate,
    Start { language: Option<String> },
    Exit(i32),
}

pub fn carry_out_verb(room: &mut impl MeetingRoom, argv: &[String]) -> Result<i32, Failure> {
    let verb = argv.first().map_or("", String::as_str);
    let extra = argv.len() > 1;
    match (verb, extra) {
        ("start", false) => start(room),
        ("stop", false) => stop(room),
        ("controls", false) => controls(room),
        _ => Err(Failure::runtime(USAGE)),
    }
    .map(|()| 0)
}

fn is_number_like(value: &str) -> bool {
    let digits = value.strip_prefix('-').unwrap_or("");
    let (whole, fraction) = digits.split_once('.').unwrap_or((digits, ""));
    let numeric = |part: &str| part.chars().all(|digit| digit.is_ascii_digit());
    !digits.is_empty() && numeric(whole) && numeric(fraction) && digits != "."
}

fn names(option: &str, full: &str) -> bool {
    option.len() > 2 && full.starts_with(option)
}

fn value_of(argument: &str, following: Option<&String>) -> Option<(String, usize)> {
    if let Some((_, attached)) = argument.split_once('=') {
        return Some((attached.to_owned(), 1));
    }
    let value = following.filter(|text| !text.starts_with('-') || is_number_like(text))?;
    Some((value.clone(), 2))
}

fn option_step(argument: &str, following: Option<&String>) -> Result<(String, usize), i32> {
    let option = argument.split('=').next().unwrap_or_default();
    if option == "-h" || names(option, "--help") {
        return Err(0);
    }
    if !names(option, "--language") {
        return Err(2);
    }
    value_of(argument, following).ok_or(2)
}

fn parse_options(arguments: &[String], language: Option<String>) -> ListenCommand {
    let Some(argument) = arguments.first() else {
        return ListenCommand::Start { language };
    };
    match option_step(argument, arguments.get(1)) {
        Ok((value, used)) => parse_options(&arguments[used..], Some(value)),
        Err(code) => ListenCommand::Exit(code),
    }
}

#[must_use]
pub fn listen_command(argv: &[String]) -> ListenCommand {
    match argv.first().map(String::as_str) {
        Some("start") => parse_options(&argv[1..], None),
        _ => ListenCommand::Delegate,
    }
}
