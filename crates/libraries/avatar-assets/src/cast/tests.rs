use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn the_voice_table_keeps_exactly_the_voice_fields() -> TestResult {
    let table: Vec<Value> = serde_json::from_str(&voices_json()?)?;
    assert_eq!(table.len(), characters()?.len());
    let first = table[0].as_object().ok_or("object")?;
    let mut keys: Vec<_> = first.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["cloudVoice", "localVoice", "name"]);
    Ok(())
}

#[test]
fn the_catalog_lists_name_and_description() -> TestResult {
    let catalog: Vec<Value> = serde_json::from_str(&catalog_json()?)?;
    assert_eq!(catalog[0]["name"], "darkness-shibu");
    assert!(catalog[0]["description"].is_string());
    Ok(())
}

fn count_sharing(model: &str) -> io::Result<usize> {
    Ok(characters()?
        .iter()
        .filter(|character| character.model == model)
        .count())
}

#[test]
fn nothing_is_linked_before_the_model_exists() -> TestResult {
    let directory = tempfile::tempdir()?;
    link_models(directory.path())?;
    assert_eq!(fs::read_dir(directory.path())?.count(), 0);
    Ok(())
}

#[test]
fn characters_sharing_a_model_each_get_a_link_idempotently() -> TestResult {
    let directory = tempfile::tempdir()?;
    fs::write(directory.path().join("sakurada-fumiriya.vrm"), b"model")?;
    link_models(directory.path())?;
    link_models(directory.path())?;
    let shared = count_sharing("sakurada-fumiriya")?;
    assert!(shared > 1);
    assert_eq!(fs::read_dir(directory.path())?.count(), shared);
    Ok(())
}
