use std::io;

pub(super) fn parse_port(name: &str, text: &str) -> io::Result<u16> {
    let digits: String = text
        .trim_start()
        .trim_start_matches('+')
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits
        .parse::<u16>()
        .map_err(|_| io::Error::other(format!("{name} is not a valid port: {text:?}")))
}
