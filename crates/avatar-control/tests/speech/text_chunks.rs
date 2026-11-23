use avatar_control::speech::speech_chunks::split_into_chunks;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use serde::Deserialize;

use crate::names::TestResult;

#[derive(Deserialize)]
struct ChunkRow {
    text: String,
    maximum: usize,
    chunks: Vec<String>,
}

fn check_row(row: &ChunkRow) -> TestResult {
    let chunks = split_into_chunks(&row.text, row.maximum)?;
    assert_eq!(chunks, row.chunks, "{:?} at {}", row.text, row.maximum);
    Ok(())
}

#[test]
fn chunks_match_the_javascript_splitter() -> TestResult {
    let rows: Vec<ChunkRow> = serde_json::from_str(include_str!("fixtures/speech_chunks.json"))?;
    assert_eq!(rows.len(), 24);
    rows.iter().try_for_each(check_row)
}

proptest! {
    #[test]
    fn chunks_rebuild_the_text_within_the_limit(text in "\\PC{0,300}", maximum in 2usize..240) {
        let chunks = split_into_chunks(&text, maximum)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        prop_assert_eq!(chunks.concat(), text);
        for chunk in &chunks {
            let length = chunk.encode_utf16().count();
            prop_assert!(length > 0 && length <= maximum);
        }
    }
}
