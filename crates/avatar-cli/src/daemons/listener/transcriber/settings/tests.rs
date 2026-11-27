use std::collections::HashMap;
use std::ffi::OsString;

use avatar_transcription::scribe::RECOGNIZED_TERMS;

use super::keyterm_source;

#[test]
fn without_a_roster_the_keyterms_are_the_fixed_terms() {
    let source = keyterm_source(&HashMap::<String, OsString>::new());
    assert_eq!(source.terms(), RECOGNIZED_TERMS.as_slice());
}
