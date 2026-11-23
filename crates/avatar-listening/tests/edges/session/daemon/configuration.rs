use avatar_listening::listener::daemon::{ListenerConfiguration, read_configuration};

fn configuration_in(text: &str) -> Result<ListenerConfiguration, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    std::fs::write(directory.path().join("listener.json"), text)?;
    Ok(read_configuration(directory.path())?)
}

#[test]
fn a_full_configuration_is_read_with_unknown_keys_ignored() -> Result<(), Box<dyn std::error::Error>>
{
    let text = r#"{"purpose": "meeting", "sources": {"human": "mic", "meeting": "mon"}, "barge_in": true, "language": "pt", "identify_speakers": true}"#;
    let configuration = configuration_in(text)?;
    assert_eq!(configuration.sources.len(), 2);
    assert!(configuration.barge_in && configuration.identify_speakers);
    assert_eq!(configuration.language.as_deref(), Some("pt"));
    Ok(())
}

#[test]
fn optional_settings_default_off() -> Result<(), Box<dyn std::error::Error>> {
    let configuration = configuration_in(r#"{"sources": {"human": "mic"}}"#)?;
    assert!(!configuration.barge_in && !configuration.identify_speakers);
    assert_eq!(configuration.language, None);
    Ok(())
}

#[test]
fn a_missing_or_malformed_configuration_is_an_error() -> Result<(), Box<dyn std::error::Error>> {
    assert!(configuration_in("not json").is_err());
    assert!(read_configuration(tempfile::tempdir()?.path()).is_err());
    Ok(())
}
