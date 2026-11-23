use avatar_control::synthesis::character::{CharacterVoice, CharacterVoiced};
use avatar_control::synthesis::contract::{SpeechGenerator, SpeechRequest, SynthesisError};

use crate::harness::{Log, Recorder, TestResult, new_log, request};

fn character(name: &str, cloud: Option<&str>, local: Option<&str>) -> CharacterVoice {
    CharacterVoice {
        name: name.to_owned(),
        cloud_voice: cloud.map(str::to_owned),
        local_voice: local.map(str::to_owned),
    }
}

fn characters() -> Vec<CharacterVoice> {
    let first = character("first", Some("firstCloud"), Some("pf_first"));
    vec![
        first,
        character("second", Some("secondCloud"), Some("pm_second")),
    ]
}

fn recorder(log: &Log) -> Recorder {
    Recorder {
        label: "cloud",
        log: log.clone(),
        fails: false,
    }
}

fn stack() -> Result<(CharacterVoiced<Recorder>, Log), SynthesisError> {
    let log = new_log();
    Ok((CharacterVoiced::new(recorder(&log), characters())?, log))
}

async fn sent_by(
    voiced: &CharacterVoiced<Recorder>,
    log: &Log,
    speech: SpeechRequest,
) -> Result<String, SynthesisError> {
    voiced.synthesize(speech).await?;
    Ok(log.lock().await[0].clone())
}

#[tokio::test]
async fn first_character_is_selected_initially() -> TestResult {
    let (voiced, log) = stack()?;
    let sent = sent_by(&voiced, &log, request("oi")).await?;
    assert!(sent.contains("voice: Some(\"firstCloud\")"));
    Ok(())
}

#[tokio::test]
async fn selecting_a_character_switches_the_cloud_voice() -> TestResult {
    let (mut voiced, log) = stack()?;
    assert!(voiced.select_character("second"));
    let sent = sent_by(&voiced, &log, request("oi")).await?;
    assert!(sent.contains("voice: Some(\"secondCloud\")"));
    Ok(())
}

#[tokio::test]
async fn an_unknown_character_keeps_the_selected_voice() -> TestResult {
    let (mut voiced, log) = stack()?;
    assert!(voiced.select_character("second"));
    assert!(!voiced.select_character("nobody"));
    let sent = sent_by(&voiced, &log, request("oi")).await?;
    assert!(sent.contains("voice: Some(\"secondCloud\")"));
    Ok(())
}

#[tokio::test]
async fn an_explicit_voice_outranks_the_character() -> TestResult {
    let (voiced, log) = stack()?;
    let mut speech = request("oi");
    speech.voice = Some("pf_dora".to_owned());
    let sent = sent_by(&voiced, &log, speech).await?;
    assert!(sent.contains("voice: Some(\"pf_dora\")"));
    Ok(())
}

#[tokio::test]
async fn the_character_local_voice_reaches_the_generator() -> TestResult {
    let (voiced, log) = stack()?;
    let sent = sent_by(&voiced, &log, request("oi")).await?;
    assert!(sent.contains("local_voice: Some(\"pf_first\")"));
    Ok(())
}

#[tokio::test]
async fn a_character_without_a_cloud_voice_leaves_the_voice_to_the_default() -> TestResult {
    let log = new_log();
    let voiced = CharacterVoiced::new(recorder(&log), vec![character("bare", None, None)])?;
    let sent = sent_by(&voiced, &log, request("oi")).await?;
    assert!(sent.contains("voice: None"));
    Ok(())
}

#[test]
fn an_empty_character_list_is_rejected() {
    assert!(matches!(
        CharacterVoiced::new(recorder(&new_log()), Vec::new()),
        Err(SynthesisError::NoCharacters)
    ));
}
