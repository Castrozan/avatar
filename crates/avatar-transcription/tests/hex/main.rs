mod pure;
#[path = "../support/mod.rs"]
mod support;

use avatar_transcription::hex::{
    HexModel, hex_discovery_file, installed_model, transcription_path,
};
use avatar_transcription::segment::{clean_transcript, wav_bytes};
use serde_json::Value;
use std::error::Error;
use std::path::Path;
use support::case::{Case, decode_hex};
use support::compare::mismatches;
use support::flow::replay;

const FLOWS: &str = include_str!("fixtures/flows.json");

#[tokio::test]
async fn hex_requests_and_errors_match_python_capture() -> Result<(), Box<dyn Error>> {
    let cases: Vec<Case> = serde_json::from_str(FLOWS)?;
    assert_eq!(cases.len(), 8);
    let replays = futures_util::future::try_join_all(cases.iter().map(replay)).await?;
    let failures: Vec<String> = cases
        .iter()
        .zip(&replays)
        .flat_map(|(case, replayed)| mismatches(case, replayed))
        .collect();
    assert!(failures.is_empty(), "{failures:#?}");
    Ok(())
}
