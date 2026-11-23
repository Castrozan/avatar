pub(super) fn parse_wait(text: &str) -> Result<f64, String> {
    let trimmed = text.trim();
    if !underscores_between_digits(trimmed) {
        return Err(format!("invalid float value: {text:?}"));
    }
    trimmed
        .replace('_', "")
        .parse::<f64>()
        .map_err(|_| format!("invalid float value: {text:?}"))
}

fn underscores_between_digits(text: &str) -> bool {
    let characters: Vec<char> = text.chars().collect();
    characters
        .iter()
        .enumerate()
        .all(|(index, character)| *character != '_' || surrounded_by_digits(&characters, index))
}

fn surrounded_by_digits(characters: &[char], index: usize) -> bool {
    let before = index.checked_sub(1).and_then(|at| characters.get(at));
    let after = characters.get(index + 1);
    before.is_some_and(char::is_ascii_digit) && after.is_some_and(char::is_ascii_digit)
}
