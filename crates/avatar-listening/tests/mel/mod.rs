use crate::support::{Outcome, count, hex_words, lcg_samples, replay_cases, rows, text};
use avatar_listening::mel::{MelFeatures, log_mel_features};
use serde_json::Value;

mod properties;

const SAMPLE_SCALE: f32 = 32_768.0;

fn samples_of(spec: &Value) -> Result<Vec<i16>, Box<dyn std::error::Error>> {
    let samples = count(&spec["samples"])?;
    match text(&spec["kind"])? {
        "zeros" => Ok(vec![0; samples]),
        "constant" => Ok(vec![
            i16::try_from(spec["value"].as_i64().ok_or("value")?)?;
            samples
        ]),
        _ => lcg_samples(spec["seed"].as_u64().ok_or("seed")?, samples),
    }
}

pub(super) fn to_audio(samples: &[i16]) -> Vec<f32> {
    samples
        .iter()
        .map(|&sample| f32::from(sample) / SAMPLE_SCALE)
        .collect()
}

fn hex_column(value: &Value) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    rows(value)?
        .iter()
        .map(|word| Ok(u32::from_str_radix(text(word)?, 16)?))
        .collect()
}

fn column_values(features: &MelFeatures, frame: usize) -> Vec<f32> {
    (0..80).map(|band| features.value(band, frame)).collect()
}

fn bits_of(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

fn check_column(
    expected: Option<&Value>,
    features: &MelFeatures,
    frame: usize,
    name: &str,
) -> Outcome {
    if let Some(words) = expected {
        assert_eq!(
            bits_of(&column_values(features, frame)),
            hex_column(words)?,
            "{name} frame {frame}"
        );
    }
    Ok(())
}

fn check_columns(case: &Value, features: &MelFeatures, name: &str) -> Outcome {
    check_column(case.get("first_column_bits"), features, 0, name)?;
    check_column(
        case.get("last_column_bits"),
        features,
        features.frames - 1,
        name,
    )
}

fn check_output(case: &Value, features: &MelFeatures, name: &str) -> Outcome {
    if let Some(hex) = case.get("output_bits").and_then(Value::as_str) {
        assert_eq!(bits_of(&features.values), hex_words(hex)?, "{name} output");
    }
    Ok(())
}

fn replay(case: &Value) -> Outcome {
    let name = format!("{:?}", case["spec"]);
    let features = log_mel_features(&to_audio(&samples_of(&case["spec"])?)).ok_or("no features")?;
    assert_eq!(features.frames, count(&case["frames"])?, "{name} frames");
    check_output(case, &features, &name)?;
    check_columns(case, &features, &name)
}

#[test]
fn every_oracle_case_matches_the_numpy_log_mel_bit_for_bit() -> Outcome {
    replay_cases("mel/fixtures/log_mel.json", "cases", 11, replay)
}

#[test]
fn inputs_shorter_than_one_hop_have_no_features() {
    assert!(log_mel_features(&[0.0; 159]).is_none());
}
