#[path = "../support/mod.rs"]
mod support;

use avatar_transcription::scribe::{RECOGNIZED_TERMS, keyterms, multipart_body, scribe_fields};
use serde_json::Value;
use std::error::Error;
use support::case::{Case, decode_hex};
use support::compare::mismatches;
use support::flow::replay;

const PURE: &str = include_str!("fixtures/pure.json");
const FLOWS: &str = include_str!("fixtures/flows.json");

fn pure_rows(section: &str) -> Vec<Value> {
    let document: Value = serde_json::from_str(PURE).unwrap_or_default();
    document[section].as_array().cloned().unwrap_or_default()
}

fn pairs(value: &Value) -> Vec<(String, String)> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(pair_of)
        .collect()
}

fn pair_of(pair: &Value) -> (String, String) {
    (
        pair[0].as_str().unwrap_or_default().to_owned(),
        pair[1].as_str().unwrap_or_default().to_owned(),
    )
}

#[test]
fn multipart_body_matches_python_bytes() {
    let rows = pure_rows("multipart");
    assert_eq!(rows.len(), 2);
    let expected: Vec<Vec<u8>> = rows
        .iter()
        .map(|row| decode_hex(row.as_str().unwrap_or_default()))
        .collect();
    assert_eq!(multipart_fixtures().to_vec(), expected);
}

fn multipart_fixtures() -> [Vec<u8>; 2] {
    let fields = vec![
        ("model_id".to_owned(), "scribe_v2".to_owned()),
        ("keyterms".to_owned(), "Hex".to_owned()),
    ];
    [
        multipart_body("BOUNDARY", &fields, b"RIFF"),
        multipart_body("B", &[], b""),
    ]
}

#[test]
fn scribe_fields_match_python_with_fixed_terms() {
    let rows = pure_rows("fields");
    assert_eq!(rows.len(), 2);
    for (language, row) in [None, Some("pt")].into_iter().zip(&rows) {
        assert_eq!(
            scribe_fields(language, &fixed_terms()),
            pairs(&row[1]),
            "language {language:?}"
        );
    }
}

fn fixed_terms() -> Vec<String> {
    RECOGNIZED_TERMS
        .iter()
        .map(|term| (*term).to_owned())
        .collect()
}

#[test]
fn keyterms_without_roster_are_the_fixed_terms() {
    assert_eq!(keyterms(None, None), fixed_terms());
}

#[tokio::test]
async fn scribe_flows_match_python_capture() -> Result<(), Box<dyn Error>> {
    let cases: Vec<Case> = serde_json::from_str(FLOWS)?;
    assert_eq!(cases.len(), 24);
    let replays = futures_util::future::try_join_all(cases.iter().map(replay)).await?;
    let failures: Vec<String> = cases
        .iter()
        .zip(&replays)
        .flat_map(|(case, replayed)| mismatches(case, replayed))
        .collect();
    assert!(failures.is_empty(), "{failures:#?}");
    Ok(())
}

#[test]
fn a_roster_line_without_a_word_is_skipped_instead_of_raising() {
    let terms = keyterms(Some("!!! | x\nMedea | y\n"), None);
    assert_eq!(terms[RECOGNIZED_TERMS.len()..], ["Medea".to_owned()]);
}
