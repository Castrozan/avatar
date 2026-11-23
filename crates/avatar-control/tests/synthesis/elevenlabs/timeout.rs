use std::time::Duration;

use proptest::prelude::*;

use avatar_control::synthesis::media_speech::elevenlabs::request_timeout;

#[test]
fn timeout_follows_the_javascript_formula() {
    assert_eq!(request_timeout("oi"), Duration::from_millis(15_060));
    assert_eq!(request_timeout(""), Duration::from_secs(15));
    assert_eq!(request_timeout(&"a".repeat(4_000)), Duration::from_mins(2));
}

#[test]
fn timeout_counts_utf16_code_units() {
    assert_eq!(request_timeout("é😀"), Duration::from_millis(15_090));
}

proptest! {
    #[test]
    fn timeout_is_bounded_and_never_shrinks_with_longer_text(text in "\\PC{0,5000}", extra in "\\PC{0,50}") {
        let longer = format!("{text}{extra}");
        let shorter_timeout = request_timeout(&text);
        prop_assert!(shorter_timeout >= Duration::from_secs(15));
        prop_assert!(shorter_timeout <= Duration::from_mins(2));
        prop_assert!(request_timeout(&longer) >= shorter_timeout);
    }
}
