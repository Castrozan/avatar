pub(crate) fn list_repr(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|item| string_repr(item)).collect();
    format!("[{}]", quoted.join(", "))
}

fn quote_for(text: &str) -> char {
    if text.contains('\'') && !text.contains('"') {
        '"'
    } else {
        '\''
    }
}

fn string_repr(text: &str) -> String {
    let quote = quote_for(text);
    let body: String = text.chars().flat_map(|ch| escaped(ch, quote)).collect();
    format!("{quote}{body}{quote}")
}

fn escaped(ch: char, quote: char) -> Vec<char> {
    match ch {
        '\\' => vec!['\\', '\\'],
        '\n' => vec!['\\', 'n'],
        '\r' => vec!['\\', 'r'],
        '\t' => vec!['\\', 't'],
        _ if ch == quote => vec!['\\', ch],
        _ => vec![ch],
    }
}
