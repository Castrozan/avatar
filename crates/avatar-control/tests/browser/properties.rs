use avatar_control::browser::{PageTarget, select_page_target};
use proptest::collection::vec;
use proptest::option;
use proptest::prelude::*;

use super::support::target;

const TYPES: [&str; 3] = ["page", "iframe", "service_worker"];
const URLS: [&str; 5] = [
    "http://localhost:3000/?output=1",
    "http://localhost:3000/controls.html",
    "http://a/",
    "https://x.test/?output=1",
    "",
];

fn build(specs: &[(usize, Option<usize>)]) -> Vec<PageTarget> {
    specs
        .iter()
        .enumerate()
        .map(|(position, (type_index, url_index))| {
            let url = url_index.map(|index| URLS[index]);
            target(&format!("t{position}"), TYPES[*type_index], url)
        })
        .collect()
}

fn is_candidate(candidate: &PageTarget, fragment: &str) -> bool {
    candidate.target_type == "page"
        && candidate
            .url
            .as_deref()
            .is_some_and(|url| url.contains(fragment))
}

proptest! {
    #[test]
    fn selects_the_first_page_whose_url_contains_the_fragment(
        specs in vec((0..TYPES.len(), option::of(0..URLS.len())), 0..10),
        fragment_index in 0..URLS.len(),
    ) {
        let targets = build(&specs);
        let fragment = URLS[fragment_index];
        let expected = targets.iter().find(|candidate| is_candidate(candidate, fragment));
        let selected = select_page_target(&targets, fragment);
        prop_assert_eq!(selected.map(|found| &found.id), expected.map(|found| &found.id));
    }
}
