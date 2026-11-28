use avatar_transcript::json::{Json, JsonObject};

fn median(sorted: &[f64]) -> Option<f64> {
    let middle = sorted.len() / 2;
    let upper = *sorted.get(middle)?;
    let below = middle.checked_sub(1).and_then(|index| sorted.get(index));
    let lower = below.copied().unwrap_or(upper);
    Some(if sorted.len() % 2 == 1 {
        upper
    } else {
        lower.midpoint(upper)
    })
}

fn rounded_to_hundredths(number: f64) -> f64 {
    format!("{number:.2}").parse().unwrap_or(number)
}

fn median_json(sorted: &[f64]) -> Json {
    median(sorted).map_or(Json::Null, |found| {
        Json::Float(rounded_to_hundredths(found))
    })
}

fn answer_count(sorted: &[f64]) -> Json {
    Json::Integer(i64::try_from(sorted.len()).unwrap_or(i64::MAX))
}

fn object_of(fields: Vec<(&str, Json)>) -> Json {
    let mut object = JsonObject::default();
    for (key, value) in fields {
        object.set(key, value);
    }
    Json::Object(object)
}

pub(super) fn summary(provider: &str, latencies: &[f64]) -> Json {
    let mut sorted = latencies.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = median_json(&sorted);
    object_of(vec![
        ("provider", Json::Text(provider.to_owned())),
        ("answers", answer_count(&sorted)),
        ("median_first_audio_s", middle),
    ])
}
