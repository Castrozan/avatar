mod routes;

use std::error::Error;
use std::time::Duration;

use avatar_control::browser::TargetLocator;
use serde_json::{Value, json};

use super::support::TestResult;
use super::targets::{TargetServer, serve_targets};

const OUTPUT_FRAGMENT: &str = "localhost:3000/?output=1";

const OUTPUT_TAB: &str = "http://localhost:3000/?output=1";

const LIST: &str = "GET /json";

const OUTPUT_ROUTE: &str = "PUT /json/new?http%3A%2F%2Flocalhost%3A3000%2F%3Foutput%3D1";

fn lines(entries: &[&str]) -> Vec<String> {
    entries.iter().map(|entry| (*entry).to_owned()).collect()
}

fn page(id: &str, url: &str) -> Value {
    json!({ "id": id, "type": "page", "url": url })
}

async fn find_or_open(server: &TargetServer) -> Result<Option<String>, Box<dyn Error>> {
    let locator = TargetLocator::new(server.port);
    let found = locator
        .find_or_open_page_target(OUTPUT_FRAGMENT, OUTPUT_TAB, Duration::ZERO)
        .await?;
    Ok(found.map(|target| target.id))
}

#[tokio::test]
async fn find_page_target_reads_the_devtools_list() -> TestResult {
    let controls = page("controls", "http://localhost:3000/controls.html");
    let output = page("output", OUTPUT_TAB);
    let server = serve_targets(vec![controls, output], None, "").await?;
    let found = TargetLocator::new(server.port)
        .find_page_target(OUTPUT_FRAGMENT)
        .await?;
    assert_eq!(found.map(|target| target.id), Some("output".to_owned()));
    assert_eq!(server.requests(), lines(&[LIST]));
    Ok(())
}

#[tokio::test]
async fn find_or_open_reuses_an_existing_output_tab() -> TestResult {
    let server = serve_targets(vec![page("o", OUTPUT_TAB)], None, "").await?;
    assert_eq!(find_or_open(&server).await?, Some("o".to_owned()));
    assert_eq!(server.requests(), lines(&[LIST]));
    Ok(())
}

#[tokio::test]
async fn find_or_open_opens_a_missing_output_tab() -> TestResult {
    let server =
        serve_targets(Vec::new(), Some(page("new", OUTPUT_TAB)), r#"{"id":"new"}"#).await?;
    assert_eq!(find_or_open(&server).await?, Some("new".to_owned()));
    assert_eq!(server.requests(), lines(&[LIST, OUTPUT_ROUTE, LIST]));
    Ok(())
}

#[tokio::test]
async fn find_or_open_reports_none_when_the_tab_never_appears() -> TestResult {
    let server = serve_targets(Vec::new(), None, "raw text").await?;
    assert_eq!(find_or_open(&server).await?, None);
    assert_eq!(server.requests(), lines(&[LIST, OUTPUT_ROUTE, LIST]));
    Ok(())
}
