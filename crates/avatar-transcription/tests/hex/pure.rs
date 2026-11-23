const PURE: &str = include_str!("fixtures/pure.json");
use super::*;

fn pure_rows(section: &str) -> Vec<Value> {
    let document: Value = serde_json::from_str(PURE).unwrap_or_default();
    document[section].as_array().cloned().unwrap_or_default()
}

fn text_at(row: &Value, index: usize) -> &str {
    row[index].as_str().unwrap_or_default()
}

fn failures_of(section: &str, check: impl Fn(&Value) -> Option<String>) -> Vec<String> {
    pure_rows(section).iter().filter_map(check).collect()
}

#[test]
fn clean_transcript_matches_python_on_every_fixture() {
    let rows = pure_rows("clean");
    assert_eq!(rows.len(), 16);
    let failures = failures_of("clean", |row| {
        let produced = clean_transcript(text_at(row, 0));
        (produced != text_at(row, 1)).then(|| format!("{row} gave {produced:?}"))
    });
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn transcription_path_matches_python_urlencode() {
    let rows = pure_rows("paths");
    assert_eq!(rows.len(), 5);
    let failures = failures_of("paths", |row| {
        let language = row[1].as_str();
        let produced = transcription_path(text_at(row, 0), language);
        (produced != text_at(row, 2)).then(|| format!("{row} gave {produced}"))
    });
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn wav_bytes_match_python_wave_module() {
    let inputs = pure_rows("wav");
    let outputs = pure_rows("wav_bytes");
    assert_eq!(inputs.len(), 4);
    let failures: Vec<String> = inputs
        .iter()
        .zip(&outputs)
        .filter_map(|(input, output)| wav_mismatch(input, output))
        .collect();
    assert!(failures.is_empty(), "{failures:#?}");
}

fn wav_mismatch(input: &Value, output: &Value) -> Option<String> {
    let produced = wav_bytes(&decode_hex(input.as_str().unwrap_or_default()));
    let expected = decode_hex(output.as_str().unwrap_or_default());
    (produced != expected).then(|| format!("{input} gave {produced:02x?}"))
}

#[test]
fn hex_discovery_file_matches_python_per_platform() {
    let platforms = ["darwin", "linux", "freebsd"];
    let rows = pure_rows("discovery");
    assert_eq!(rows.len(), platforms.len());
    let failures: Vec<String> = platforms
        .iter()
        .zip(&rows)
        .filter_map(|(platform, row)| discovery_mismatch(platform, row))
        .collect();
    assert!(failures.is_empty(), "{failures:#?}");
}

fn discovery_mismatch(platform: &str, row: &Value) -> Option<String> {
    let produced = hex_discovery_file(Path::new("HOME"), platform);
    let expected = row.as_str().unwrap_or_default();
    (produced.to_string_lossy() != expected).then(|| format!("{platform}: {}", produced.display()))
}

fn installed_outcome(models: &Value) -> String {
    let parsed: Vec<HexModel> = serde_json::from_value(models.clone()).unwrap_or_default();
    match installed_model(&parsed) {
        Ok(model) => model.to_owned(),
        Err(error) => format!("ConnectionError: {error}"),
    }
}

#[test]
fn installed_model_matches_python_selection() {
    let rows = pure_rows("installed");
    assert_eq!(rows.len(), 5);
    let failures = failures_of("installed", |row| {
        let produced = installed_outcome(&row[0]);
        (produced != text_at(row, 1)).then(|| format!("{row} gave {produced}"))
    });
    assert!(failures.is_empty(), "{failures:#?}");
}
