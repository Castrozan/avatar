use std::path::Path;

use avatar_control::site::{percent_decode, quote, translate_path};
use proptest::prelude::*;

proptest! {
    #[test]
    fn quoting_then_decoding_restores_any_text(text in "\\PC{0,40}") {
        prop_assert_eq!(percent_decode(&quote(&text)), text);
    }

    #[test]
    fn translated_paths_never_leave_the_root(path in "[a-z./%0-9A-F]{0,40}") {
        let root = Path::new("/srv/renderer");
        prop_assert!(translate_path(root, &path).starts_with(root));
    }

    #[test]
    fn decoding_never_grows_the_text(text in "[ -~]{0,40}") {
        prop_assert!(percent_decode(&text).len() <= text.len());
    }
}
