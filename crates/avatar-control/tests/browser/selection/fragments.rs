use super::super::support::target;

use super::*;

#[test]
fn fragment_may_match_in_the_middle_of_the_url() {
    let targets = [target(
        "mid",
        "page",
        Some("https://x.test/prefix-localhost:3000/?output=1-suffix"),
    )];
    assert_eq!(selected(&targets, OUTPUT).as_deref(), Some("mid"));
}

#[test]
fn fragment_longer_than_url_selects_nothing() {
    let targets = [target("short", "page", Some("http://a/"))];
    assert_eq!(selected(&targets, "http://a/extra"), None);
}

#[test]
fn port_prefix_matches_a_longer_query() {
    let targets = [target(
        "ten",
        "page",
        Some("http://localhost:3000/?output=10"),
    )];
    assert_eq!(selected(&targets, "?output=1").as_deref(), Some("ten"));
}

#[test]
fn non_page_target_without_url_is_ignored() {
    let targets = [
        target("bare", "browser", None),
        target("p", "page", Some("http://localhost:3000/?output=1")),
    ];
    assert_eq!(selected(&targets, OUTPUT).as_deref(), Some("p"));
}

#[test]
fn unicode_fragment_matches_its_url() {
    let targets = [
        target("uni", "page", Some("http://localhost:3000/?t=é")),
        target("u2", "page", Some("http://localhost:3000/?t=é")),
    ];
    assert_eq!(selected(&targets, "t=é").as_deref(), Some("uni"));
}

#[test]
fn empty_fragment_matches_the_first_page() {
    let targets = [
        target("only", "page", Some("http://x/")),
        target("second", "page", Some("http://y/")),
    ];
    assert_eq!(selected(&targets, "").as_deref(), Some("only"));
}
