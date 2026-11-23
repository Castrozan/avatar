mod fragments;

use avatar_control::browser::{PageTarget, select_page_target};

use super::support::target;

const OUTPUT: &str = "localhost:3000/?output=1";

fn selected(targets: &[PageTarget], fragment: &str) -> Option<String> {
    select_page_target(targets, fragment).map(|found| found.id.clone())
}

#[test]
fn single_matching_page_is_selected() {
    let targets = [target("a", "page", Some("http://localhost:3000/?output=1"))];
    assert_eq!(selected(&targets, OUTPUT).as_deref(), Some("a"));
}

#[test]
fn control_window_is_skipped_for_the_output_page() {
    let targets = [
        target(
            "controls",
            "page",
            Some("http://localhost:3000/controls.html"),
        ),
        target("output", "page", Some("http://localhost:3000/?output=1")),
    ];
    assert_eq!(selected(&targets, OUTPUT).as_deref(), Some("output"));
}

#[test]
fn first_matching_page_wins() {
    let targets = [
        target("first", "page", Some("http://localhost:3000/?output=1")),
        target(
            "second",
            "page",
            Some("http://localhost:3000/?output=1&x=2"),
        ),
    ];
    assert_eq!(selected(&targets, OUTPUT).as_deref(), Some("first"));
}

#[test]
fn service_worker_with_matching_url_is_skipped() {
    let targets = [
        target(
            "worker",
            "service_worker",
            Some("http://localhost:3000/?output=1"),
        ),
        target("page", "page", Some("http://localhost:3000/?output=1")),
    ];
    assert_eq!(selected(&targets, OUTPUT).as_deref(), Some("page"));
}

#[test]
fn iframe_only_list_selects_nothing() {
    let targets = [target(
        "frame",
        "iframe",
        Some("http://localhost:3000/?output=1"),
    )];
    assert_eq!(selected(&targets, OUTPUT), None);
}

#[test]
fn list_without_match_selects_nothing() {
    let targets = [target("other", "page", Some("http://example.com/"))];
    assert_eq!(selected(&targets, OUTPUT), None);
}

#[test]
fn empty_list_selects_nothing() {
    assert_eq!(selected(&[], "anything"), None);
}

#[test]
fn target_type_comparison_is_case_sensitive() {
    let targets = [target(
        "upper",
        "Page",
        Some("http://localhost:3000/?output=1"),
    )];
    assert_eq!(selected(&targets, OUTPUT), None);
}
