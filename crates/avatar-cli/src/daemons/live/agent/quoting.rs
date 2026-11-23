pub(super) fn shell_quote(argument: &str) -> String {
    let safe =
        |character: char| character.is_ascii_alphanumeric() || "_@%+=:,./-".contains(character);
    if !argument.is_empty() && argument.chars().all(safe) {
        return argument.to_owned();
    }
    format!("'{}'", argument.replace('\'', "'\"'\"'"))
}
