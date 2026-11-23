use std::error::Error;
use std::num::TryFromIntError;

use avatar_transcript::json::{self, Json, JsonObject};
use avatar_transcript::transcript::store::FileTranscriptStore;
use avatar_transcript::transcript::{Transcript, Utterance};
use proptest::prelude::*;
use serde_json::Value;

proptest! {
    #[test]
    fn encoded_text_parses_back_to_the_same_text(text in any::<String>()) {
        let encoded = json::encode(&Json::Text(text.clone()));
        let parsed: Value = serde_json::from_str(&encoded).map_err(|error| TestCaseError::fail(error.to_string()))?;
        prop_assert_eq!(parsed, Value::String(text));
    }

    #[test]
    fn encoded_float_parses_back_to_the_same_bits(number in any::<f64>()) {
        prop_assume!(number.is_finite());
        let encoded = json::python_float(number);
        let parsed: f64 = encoded.parse().map_err(|_| TestCaseError::fail(encoded.clone()))?;
        prop_assert_eq!(parsed.to_bits(), number.to_bits());
    }

    #[test]
    fn encoded_float_is_always_a_python_float_literal(number in any::<f64>()) {
        prop_assume!(number.is_finite());
        let encoded = json::python_float(number);
        prop_assert!(encoded.contains('.') || encoded.contains('e'));
    }

    #[test]
    fn appended_sequences_count_up_and_reads_return_the_newer_suffix(
        texts in prop::collection::vec(any::<String>(), 1..12),
        cut in 0i64..12,
    ) {
        check_sequences(&texts, cut).map_err(|error| TestCaseError::fail(error.to_string()))?;
    }
}

fn check_sequences(texts: &[String], cut: i64) -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let transcript = Transcript::new(FileTranscriptStore::new(
        directory.path().join("transcript.jsonl"),
    ));
    append_texts(&transcript, texts)?;
    assert_eq!(
        newer_sequences(&transcript, cut)?,
        expected_sequences(texts.len(), cut)?
    );
    Ok(())
}

fn append_texts(
    transcript: &Transcript<FileTranscriptStore>,
    texts: &[String],
) -> Result<(), Box<dyn Error>> {
    for (index, text) in texts.iter().enumerate() {
        let entry = transcript.append(utterance(text), 1.5, &JsonObject::default())?;
        assert!(json::encode(&entry).contains(&format!("\"sequence\": {}", index + 1)));
    }
    Ok(())
}

fn utterance(text: &str) -> Utterance {
    Utterance {
        source: "meeting".to_owned(),
        text: text.to_owned(),
        speaker: None,
    }
}

fn newer_sequences(
    transcript: &Transcript<FileTranscriptStore>,
    cut: i64,
) -> Result<Vec<i64>, Box<dyn Error>> {
    Ok(transcript
        .read_since(cut)?
        .iter()
        .filter_map(|entry| entry["sequence"].as_i64())
        .collect())
}

fn expected_sequences(count: usize, cut: i64) -> Result<Vec<i64>, TryFromIntError> {
    Ok((1..=i64::try_from(count)?)
        .filter(|sequence| *sequence > cut)
        .collect())
}
