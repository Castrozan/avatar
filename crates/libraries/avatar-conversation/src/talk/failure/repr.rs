use crate::talk::ports::Invocation;

#[must_use]
pub fn python_text_repr(text: &str) -> String {
    let quote = if text.contains('\'') && !text.contains('"') {
        '"'
    } else {
        '\''
    };
    let body: String = text.chars().flat_map(|ch| escaped(ch, quote)).collect();
    format!("{quote}{body}{quote}")
}

fn escaped(character: char, quote: char) -> Vec<char> {
    match character {
        '\\' => vec!['\\', '\\'],
        '\n' => vec!['\\', 'n'],
        '\r' => vec!['\\', 'r'],
        '\t' => vec!['\\', 't'],
        other if other == quote => vec!['\\', other],
        other if other.is_control() => format!("\\x{:02x}", u32::from(other)).chars().collect(),
        other => vec![other],
    }
}

fn argv_repr(argv: &[String]) -> String {
    let items: Vec<String> = argv.iter().map(|part| python_text_repr(part)).collect();
    format!("[{}]", items.join(", "))
}

fn seconds_text(seconds: f64) -> String {
    if seconds.fract() == 0.0 {
        return format!("{seconds:.0}");
    }
    seconds.to_string()
}

pub(super) fn timed_out(invocation: &Invocation) -> String {
    let seconds = invocation
        .timeout_seconds
        .map_or_else(String::new, seconds_text);
    format!(
        "Command '{}' timed out after {seconds} seconds",
        argv_repr(&invocation.argv)
    )
}

pub(super) fn non_zero_exit(argv: &[String], status: i32) -> String {
    format!(
        "Command '{}' returned non-zero exit status {status}.",
        argv_repr(argv)
    )
}
