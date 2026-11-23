use avatar_transcription::spoken_languages::{
    SPOKEN_LANGUAGES, languages_named_by, spoken_language,
};
use serde_json::Value;

const LANGUAGES: &str = include_str!("fixtures/languages.json");

fn section(name: &str) -> Vec<Value> {
    let document: Value = serde_json::from_str(LANGUAGES).unwrap_or_default();
    document[name].as_array().cloned().unwrap_or_default()
}

fn names(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .map(|item| item.as_str().unwrap_or_default().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn iso_639_1_of(code: &str) -> Option<String> {
    spoken_language(code).map(|language| language.pinning_code().to_owned())
}

#[test]
fn spoken_language_lookup_matches_python_for_every_code() {
    let rows = section("languages");
    assert_eq!(rows.len(), 15);
    for row in rows {
        let code = row[0].as_str();
        let expected = row[1].as_str().map(str::to_owned);
        let produced = code.and_then(iso_639_1_of);
        assert_eq!(produced, expected, "code {code:?}");
    }
}

#[test]
fn languages_named_by_matches_python_including_absent_code() {
    for row in section("languages") {
        let produced: Vec<String> = languages_named_by(row[0].as_str())
            .iter()
            .map(|language| language.pinning_code().to_owned())
            .collect();
        assert_eq!(produced, names(&row[2]), "code {}", row[0]);
    }
}

#[test]
fn writes_matches_python_isalpha_script_check() {
    let rows = section("writes");
    assert_eq!(rows.len(), 40);
    let failures: Vec<String> = rows.iter().filter_map(writes_mismatch).collect();
    assert!(failures.is_empty(), "{failures:#?}");
}

fn writes_mismatch(row: &Value) -> Option<String> {
    let text = row[0].as_str().unwrap_or_default();
    let code = row[1].as_str().unwrap_or_default();
    let language = SPOKEN_LANGUAGES
        .into_iter()
        .find(|candidate| candidate.pinning_code() == code)?;
    let produced = language.writes(text);
    (Some(produced) != row[2].as_bool()).then(|| format!("{code} {text:?} gave {produced}"))
}
