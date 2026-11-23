use avatar_mac::client::wav_samples;
use serde_json::Value;

use crate::support::{each, hex_bytes, hex_text, parse};

fn check(case: &Value) {
    let wav = hex_bytes(case["wav"].as_str().unwrap_or_default());
    let outcome = wav_samples(&wav).map(|samples| hex_text(&samples));
    let expected = case["result"]["ok"].as_str().map(str::to_owned);
    assert_eq!(outcome.ok(), expected, "{}", case["wav"]);
}

#[test]
fn recordings_yield_the_frames_python_reads_or_fail_where_python_fails() {
    each(
        &parse(include_str!("../fixtures/client.json"))["wavs"],
        check,
    );
}
