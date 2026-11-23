use super::addressee::current;
use crate::talk::failure::{TalkFailure, python_text_repr};
use crate::talk::ports::Files;
use crate::utterances;

const HEARD_FILE: &str = "heard";

fn heard_through(files: &mut impl Files) -> Result<u64, TalkFailure> {
    let Some(text) = files.read(HEARD_FILE) else {
        return Ok(0);
    };
    text.trim().parse::<u64>().map_err(|_| {
        TalkFailure::new(format!(
            "invalid literal for int() with base 10: {}",
            python_text_repr(&text)
        ))
    })
}

pub fn last_heard(files: &mut impl Files) -> Result<u64, TalkFailure> {
    let heard = heard_through(files)?;
    let addressed = current(files)?.map(|addressee| u64::try_from(addressee.heard_through));
    Ok(utterances::last_heard(
        heard,
        addressed.and_then(Result::ok),
    ))
}

pub fn remember(files: &mut impl Files, sequence: u64) {
    files.write(HEARD_FILE, &sequence.to_string());
}
