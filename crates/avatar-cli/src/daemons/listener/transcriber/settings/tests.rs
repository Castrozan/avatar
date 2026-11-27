use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::Path;

use avatar_transcription::scribe::RECOGNIZED_TERMS;

use super::{api_key, keyterm_source};

type TestResult = Result<(), Box<dyn Error>>;

fn write_key_file(home: &Path, contents: &str) -> TestResult {
    fs::create_dir_all(home.join(".secrets"))?;
    fs::write(home.join(".secrets/elevenlabs-api-key"), contents)?;
    Ok(())
}

fn key_chosen(variable: Option<&str>, file: Option<&str>) -> Result<String, Box<dyn Error>> {
    let home = tempfile::tempdir()?;
    if let Some(contents) = file {
        write_key_file(home.path(), contents)?;
    }
    let mut environment: HashMap<String, OsString> =
        HashMap::from([("HOME".to_owned(), home.path().into())]);
    environment.extend(variable.map(|key| ("ELEVENLABS_API_KEY".to_owned(), key.into())));
    Ok(api_key(&environment))
}

const KEY_CASES: [(Option<&str>, Option<&str>, &str); 6] = [
    (None, None, ""),
    (None, Some("  file-key\n"), "file-key"),
    (Some(""), Some("file-key\n"), "file-key"),
    (Some("environment-key"), Some("file-key"), "environment-key"),
    (Some("environment-key"), None, "environment-key"),
    (Some(""), None, ""),
];

#[test]
fn the_api_key_is_the_environment_variable_else_the_trimmed_file_else_empty() -> TestResult {
    for (variable, file, expected) in KEY_CASES {
        let chosen = key_chosen(variable, file)?;
        assert_eq!(chosen, expected, "{variable:?} {file:?}");
    }
    Ok(())
}

#[test]
fn without_a_home_the_api_key_is_empty() {
    assert_eq!(api_key(&HashMap::<String, OsString>::new()), "");
}

#[test]
fn without_a_roster_the_keyterms_are_the_fixed_terms() {
    let source = keyterm_source(&HashMap::<String, OsString>::new());
    assert_eq!(source.terms(), RECOGNIZED_TERMS.as_slice());
}
