use clap::Error;
use clap::error::ErrorKind;

fn all_digits(text: &str) -> bool {
    text.chars().all(|character| character.is_ascii_digit())
}

fn argparse_negative_number(token: &str) -> bool {
    let Some(digits) = token.strip_prefix('-') else {
        return false;
    };
    match digits.split_once('.') {
        None => !digits.is_empty() && all_digits(digits),
        Some((whole, fraction)) => {
            all_digits(whole) && !fraction.is_empty() && all_digits(fraction)
        }
    }
}

fn unclassifiable(token: &str) -> bool {
    let single_dash = token.starts_with('-') && !token.starts_with("--");
    single_dash && token.parse::<f64>().is_ok() && !argparse_negative_number(token)
}

pub(super) fn reject_unclassifiable_numbers(arguments: &[String]) -> Result<(), Error> {
    let before_separator = arguments.iter().take_while(|token| token.as_str() != "--");
    match before_separator
        .map(String::as_str)
        .find(|token| unclassifiable(token))
    {
        Some(token) => Err(Error::raw(
            ErrorKind::UnknownArgument,
            format!("unrecognized arguments: {token}\n"),
        )),
        None => Ok(()),
    }
}
