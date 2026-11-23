use avatar_control::browser::TargetLocator;

use super::super::support::TestResult;
use super::super::targets::serve_targets;

use super::*;

async fn assert_route(tab_url: &str, expected_route: &str) -> TestResult {
    let server = serve_targets(Vec::new(), None, "").await?;
    TargetLocator::new(server.port).open_tab(tab_url).await?;
    assert_eq!(server.requests(), vec![format!("PUT {expected_route}")]);
    Ok(())
}

#[tokio::test]
async fn open_tab_encodes_query_and_keeps_its_separators() -> TestResult {
    assert_route(
        "http://localhost:3000/?output=1&view=side",
        "/json/new?http%3A%2F%2Flocalhost%3A3000%2F%3Foutput%3D1%26view%3Dside",
    )
    .await
}

#[tokio::test]
async fn open_tab_encodes_plain_output_url() -> TestResult {
    assert_route(
        OUTPUT_TAB,
        "/json/new?http%3A%2F%2Flocalhost%3A3000%2F%3Foutput%3D1",
    )
    .await
}

#[tokio::test]
async fn open_tab_encodes_spaces_and_non_ascii_bytes() -> TestResult {
    assert_route("a b/c?d=é", "/json/new?a%20b%2Fc%3Fd%3D%C3%A9").await
}

#[tokio::test]
async fn open_tab_leaves_unreserved_marks_untouched() -> TestResult {
    assert_route("!'()*~-_.", "/json/new?!'()*~-_.").await
}

#[tokio::test]
async fn open_tab_with_empty_url_sends_empty_query() -> TestResult {
    assert_route("", "/json/new?").await
}

#[tokio::test]
async fn open_tab_encodes_emoji_and_fragment() -> TestResult {
    assert_route(
        "https://example.test/🎥?x=y&z=1#frag",
        "/json/new?https%3A%2F%2Fexample.test%2F%F0%9F%8E%A5%3Fx%3Dy%26z%3D1%23frag",
    )
    .await
}

#[tokio::test]
async fn open_tab_encodes_plus_and_percent_signs() -> TestResult {
    assert_route("a+b=c%20d", "/json/new?a%2Bb%3Dc%2520d").await
}
