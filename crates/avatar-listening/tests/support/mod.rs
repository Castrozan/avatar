use std::error::Error;
use std::path::Path;

use serde_json::Value;

pub(crate) type Outcome = Result<(), Box<dyn Error>>;

const MASK_STATE_BITS: u64 = u64::MAX;

pub(crate) fn fixture(relative: &str) -> Result<Value, Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(relative);
    let text = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

pub(crate) fn rows(value: &Value) -> Result<&[Value], Box<dyn Error>> {
    let array = value.as_array().ok_or("expected a JSON array")?;
    Ok(array.as_slice())
}

pub(crate) fn text(value: &Value) -> Result<&str, Box<dyn Error>> {
    Ok(value.as_str().ok_or("expected a JSON string")?)
}

pub(crate) fn number(value: &Value) -> Result<f64, Box<dyn Error>> {
    Ok(value.as_f64().ok_or("expected a JSON number")?)
}

pub(crate) fn flag(value: &Value) -> Result<bool, Box<dyn Error>> {
    Ok(value.as_bool().ok_or("expected a JSON boolean")?)
}

pub(crate) fn count(value: &Value) -> Result<usize, Box<dyn Error>> {
    Ok(usize::try_from(
        value.as_u64().ok_or("expected a JSON count")?,
    )?)
}

pub(crate) fn optional_count(value: &Value) -> Result<Option<usize>, Box<dyn Error>> {
    if value.is_null() {
        return Ok(None);
    }
    Ok(Some(count(value)?))
}

pub(crate) fn optional_text(value: &Value) -> Result<Option<&str>, Box<dyn Error>> {
    if value.is_null() {
        return Ok(None);
    }
    Ok(Some(text(value)?))
}

pub(crate) fn hex_words(hex: &str) -> Result<Vec<u32>, Box<dyn Error>> {
    let mut words = Vec::new();
    for chunk in hex.as_bytes().chunks(8) {
        words.push(u32::from_str_radix(std::str::from_utf8(chunk)?, 16)?);
    }
    Ok(words)
}

pub(crate) fn lcg_samples(seed: u64, length: usize) -> Result<Vec<i16>, Box<dyn Error>> {
    let mut state = seed & MASK_STATE_BITS;
    let mut samples = Vec::with_capacity(length);
    for _ in 0..length {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let offset = i64::try_from((state >> 33) % 20_001)?;
        samples.push(i16::try_from(offset - 10_000)?);
    }
    Ok(samples)
}

pub(crate) fn pcm_of(samples: &[i16]) -> Vec<u8> {
    samples
        .iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect()
}

pub(crate) fn replay_cases(
    relative: &str,
    key: &str,
    expected: usize,
    replay: fn(&Value) -> Outcome,
) -> Outcome {
    let document = fixture(relative)?;
    let cases = rows(&document[key])?;
    assert_eq!(cases.len(), expected);
    cases.iter().try_for_each(replay)
}
