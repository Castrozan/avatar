use avatar_transcript::echo::is_python_space;
use regex::Regex;

const ANSWER_MARKER: &str = "\u{23fa}";
const LINE_BREAKS: [char; 10] = [
    '\n', '\r', '\u{b}', '\u{c}', '\u{1c}', '\u{1d}', '\u{1e}', '\u{85}', '\u{2028}', '\u{2029}',
];

fn is_line_break(character: char) -> bool {
    LINE_BREAKS.contains(&character)
}

#[must_use]
pub fn python_lines(text: &str) -> Vec<String> {
    let unified = text.replace("\r\n", "\n");
    let mut lines: Vec<String> = unified.split(is_line_break).map(str::to_owned).collect();
    lines.pop_if(|last| last.is_empty());
    lines
}

fn stripped(line: &str) -> &str {
    line.trim_matches(is_python_space)
}

fn stripped_lines(output: &str) -> Vec<String> {
    let lines = python_lines(output);
    let kept = lines
        .iter()
        .map(|line| stripped(line))
        .filter(|line| !line.is_empty());
    kept.map(str::to_owned).collect()
}

#[must_use]
pub fn spoken_answer(output: &str) -> (Option<String>, String) {
    let lines = stripped_lines(output);
    let marked = lines
        .iter()
        .rev()
        .find_map(|line| line.strip_prefix(ANSWER_MARKER));
    (
        marked.map(|rest| stripped(rest).to_owned()),
        lines.last().cloned().unwrap_or_default(),
    )
}

fn answer_pattern(tag: &str) -> Option<Regex> {
    let space = r"[\s\x1c-\x1f]";
    let pattern = format!(
        r"^\W?{space}*{}:{space}*([^\s\x1c-\x1f].*)$",
        regex::escape(tag)
    );
    Regex::new(&pattern).ok()
}

#[must_use]
pub fn tagged_answer(output: &str, tag: &str) -> Option<String> {
    let matcher = answer_pattern(tag)?;
    let lines = python_lines(output);
    let found = lines
        .iter()
        .find_map(|line| matcher.captures(stripped(line)));
    found.map(|captures| stripped(&captures[1]).to_owned())
}
