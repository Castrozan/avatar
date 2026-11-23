use std::error::Error;

use serde_json::Value;

use crate::support::{count, lcg_samples, pcm_of, rows};

pub(super) fn frame_bytes(spec: &Value) -> Result<Vec<u8>, Box<dyn Error>> {
    if spec.get("seed").is_some() {
        return seeded_frame(spec);
    }
    if spec.get("samples").is_some() {
        return sample_frame(&spec["samples"]);
    }
    raw_frame(&spec["raw_bytes"])
}

fn seeded_frame(spec: &Value) -> Result<Vec<u8>, Box<dyn Error>> {
    let seed = spec["seed"].as_u64().ok_or("seed")?;
    let samples = lcg_samples(seed, count(&spec["length"])?)?;
    Ok(pcm_of(&samples))
}

fn sample_frame(samples: &Value) -> Result<Vec<u8>, Box<dyn Error>> {
    let values = rows(samples)?
        .iter()
        .map(|value| Ok(i16::try_from(value.as_i64().ok_or("sample")?)?))
        .collect::<Result<Vec<i16>, Box<dyn Error>>>()?;
    Ok(pcm_of(&values))
}

fn raw_frame(bytes: &Value) -> Result<Vec<u8>, Box<dyn Error>> {
    rows(bytes)?
        .iter()
        .map(|value| Ok(u8::try_from(value.as_u64().ok_or("byte")?)?))
        .collect()
}
