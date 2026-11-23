use avatar_floor::quiet::{
    ARM_CHECK_SECONDS, AVATAR_SPEAKER, CALL_ENDED, CLOSING, Entry, ListenerPurpose,
    MAX_TURN_CHARACTERS, QUIET_CLOSE_SECONDS, Seat, TRANSCRIPT_POLL_SECONDS,
    UNFINISHED_TURN_SETTLE_SECONDS,
};
use serde_json::{Value, json};

const PYTHON_ENTRY: &str =
    r#"{"sequence":4,"text":"a","source":"s","speaker":null,"turn_finished":true,"time":1.5}"#;

pub(crate) fn fragment(
    sequence: i64,
    text: &str,
    speaker: Option<&str>,
    finished: Option<bool>,
) -> Result<Entry, Box<dyn std::error::Error>> {
    let value: Value = json!({
        "sequence": sequence,
        "text": text,
        "source": "mic",
        "speaker": speaker,
        "turn_finished": finished,
    });
    Ok(serde_json::from_value(value)?)
}

pub(crate) fn unfinished_fragment(
    sequence: i64,
    text: &str,
    speaker: Option<&str>,
) -> Result<Entry, Box<dyn std::error::Error>> {
    fragment(sequence, text, speaker, Some(false))
}

pub(crate) fn finished_fragment(
    sequence: i64,
    text: &str,
    speaker: Option<&str>,
) -> Result<Entry, Box<dyn std::error::Error>> {
    fragment(sequence, text, speaker, Some(true))
}

pub(crate) fn sequences(entries: &[&Entry]) -> Vec<i64> {
    entries.iter().map(|entry| entry.sequence).collect()
}

#[test]
fn entry_keeps_every_key_the_python_dict_carries() -> Result<(), Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_str(PYTHON_ENTRY)?;
    let entry: Entry = serde_json::from_value(value.clone())?;
    assert_eq!(serde_json::to_value(&entry)?, value);
    Ok(())
}

#[test]
fn seats_serialize_as_the_python_values() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(serde_json::to_value(Seat::Discord)?, json!("discord"));
    assert_eq!(serde_json::to_value(Seat::Room)?, json!("room"));
    assert_eq!(serde_json::to_value(Seat::Remote)?, json!("remote"));
    Ok(())
}

#[test]
fn listener_purposes_serialize_as_the_python_values() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        serde_json::to_value(ListenerPurpose::Conversation)?,
        json!("conversation")
    );
    assert_eq!(
        serde_json::to_value(ListenerPurpose::Meeting)?,
        json!("meeting")
    );
    assert_eq!(serde_json::to_value(ListenerPurpose::Live)?, json!("live"));
    Ok(())
}

#[test]
fn timings_match_the_python_module() {
    assert_eq!(QUIET_CLOSE_SECONDS.to_bits(), 600.0_f64.to_bits());
    assert_eq!(ARM_CHECK_SECONDS.to_bits(), 5.0_f64.to_bits());
    assert_eq!(TRANSCRIPT_POLL_SECONDS.to_bits(), 0.2_f64.to_bits());
    assert_eq!(UNFINISHED_TURN_SETTLE_SECONDS.to_bits(), 2.5_f64.to_bits());
}

#[test]
fn turn_limits_and_messages_match_the_python_module() {
    assert_eq!(MAX_TURN_CHARACTERS, 16_384);
    assert_eq!(AVATAR_SPEAKER, "avatar");
    assert_eq!(
        CLOSING,
        "closing a conversation the avatar has not spoken in for ten minutes"
    );
    assert_eq!(
        CALL_ENDED,
        "closing the conversation because the listener left the Discord call"
    );
}
