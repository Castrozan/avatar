mod dispatch;

use std::error::Error;

use avatar_transcript::transcript::report::Stamp;
use serde_json::Value;

const FIXTURE: &str = include_str!("fixtures/reports.json");
const STAMP: &str = "2026-03-04T05:06:07.123456+00:00";

fn stamp_of(case: &Value) -> Stamp {
    Stamp {
        timestamp: STAMP.to_owned(),
        room: case["room"].as_str().map(str::to_owned),
    }
}

#[test]
fn every_report_line_matches_python_byte_for_byte() -> Result<(), Box<dyn Error>> {
    let document: Value = serde_json::from_str(FIXTURE)?;
    let cases = document["cases"].as_array().ok_or("cases")?;
    assert_eq!(cases.len(), 63);
    for case in cases {
        let line = dispatch::line_of(&stamp_of(case), case)?;
        assert_eq!(Some(line.as_str()), case["line"].as_str(), "{case}");
    }
    Ok(())
}

#[test]
fn conversation_entries_without_text_or_sequence_fail_like_python_key_errors() {
    use avatar_transcript::transcript::report::report_conversation;
    let stamp = stamp_of(&Value::Null);
    let sequence_only = serde_json::json!({"sequence": 1});
    let text_only = serde_json::json!({"text": "x"});
    assert!(report_conversation(&stamp, &sequence_only).is_err());
    assert!(report_conversation(&stamp, &text_only).is_err());
}
